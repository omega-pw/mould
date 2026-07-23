use crate::log;
use async_compression::tokio::write::{BrotliEncoder, DeflateEncoder, GzipEncoder, ZstdEncoder};
use async_trait::async_trait;
use bytes::Bytes;
use futures::StreamExt;
use headers::{ContentLength, HeaderMapExt};
use hyper::body::Frame;
use hyper::body::Incoming;
use hyper::header::{CONTENT_ENCODING, CONTENT_TYPE};
use hyper::{Request, Response};
use std::net::SocketAddr;
use tihu::Handler;
use tihu::Middleware;
use tihu_native::http::body_to_stream;
use tihu_native::http::Body;
use tihu_native::http::RequestData;
use tihu_native::ErrNo;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

pub type In<Ctx> = (Ctx, Request<Incoming>, SocketAddr, RequestData);
pub type Out = Result<Response<Body>, ErrNo>;

/// 支持的压缩算法
#[derive(Clone, Copy, Debug)]
enum Encoding {
    Gzip,
    Deflate,
    Brotli,
    Zstd,
}

impl Encoding {
    fn as_header_value(&self) -> &'static str {
        match self {
            Encoding::Gzip => "gzip",
            Encoding::Deflate => "deflate",
            Encoding::Brotli => "br",
            Encoding::Zstd => "zstd",
        }
    }
}

/// 文本类型的 Content-Type 前缀，只有匹配这些前缀的响应才会被压缩
const TEXT_CONTENT_TYPES: &[&str] = &[
    "text/",
    "application/json",
    "application/xml",
    "application/javascript",
    "application/ecmascript",
    "application/x-httpd-php",
    "application/x-yaml",
    "application/x-www-form-urlencoded",
    "application/ld+json",
    "application/rss+xml",
    "application/atom+xml",
    "application/xhtml+xml",
    "application/manifest+json",
    "image/svg+xml",
];

/// 判断 Content-Type 是否为文本类型
fn is_text_content_type(content_type: Option<&str>) -> bool {
    let ct = match content_type {
        Some(ct) => ct,
        None => return false,
    };
    // 去掉可能的 charset 等参数
    let ct = ct.split(';').next().unwrap_or("").trim().to_lowercase();
    TEXT_CONTENT_TYPES
        .iter()
        .any(|prefix| ct.starts_with(prefix))
}

/// 从 Accept-Encoding 请求头中解析客户端支持的编码，按优先级排序
fn parse_accept_encoding(header_value: &str) -> Vec<Encoding> {
    let mut encodings: Vec<(Encoding, f32)> = Vec::new();

    for part in header_value.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let mut segments = part.splitn(2, ';');
        let encoding_str = segments.next().unwrap_or("").trim().to_lowercase();
        let quality = segments
            .next()
            .and_then(|s| {
                let s = s.trim();
                if s.starts_with("q=") {
                    s[2..].parse::<f32>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(1.0);

        // 忽略 q=0 的编码
        if quality <= 0.0 {
            continue;
        }

        let encoding = match encoding_str.as_str() {
            "gzip" | "x-gzip" => Some(Encoding::Gzip),
            "deflate" => Some(Encoding::Deflate),
            "br" | "brotli" => Some(Encoding::Brotli),
            "zstd" => Some(Encoding::Zstd),
            _ => None,
        };

        if let Some(enc) = encoding {
            encodings.push((enc, quality));
        }
    }

    // 按 quality 降序排序，quality 相同时保持原始顺序（稳定排序）
    encodings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    encodings.into_iter().map(|(enc, _)| enc).collect()
}

pub struct CompressHandler<H> {
    inner: H,
}

#[async_trait]
impl<H, Ctx> Handler<In<Ctx>> for CompressHandler<H>
where
    H: Handler<In<Ctx>, Out = Out>,
    Ctx: Send + Sync + 'static,
{
    type Out = H::Out;
    async fn handle(&self, (context, request, remote_addr, request_data): In<Ctx>) -> Self::Out {
        // 1. 解析客户端支持的编码
        let accept_encoding = request
            .headers()
            .get(hyper::header::ACCEPT_ENCODING)
            .and_then(|v| v.to_str().ok());

        let supported_encodings = accept_encoding
            .map(parse_accept_encoding)
            .unwrap_or_default();

        // 2. 调用内部 handler 获取响应
        let mut response = self
            .inner
            .handle((context, request, remote_addr, request_data))
            .await?;

        // 检查响应是否已有 Content-Encoding
        if response.headers().contains_key(CONTENT_ENCODING) {
            return Ok(response);
        }

        // 检查 Content-Type 是否为文本类型
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok());

        if !is_text_content_type(content_type) {
            return Ok(response);
        }

        // 如果 Content-Length 小于 256 字节，则不压缩
        let content_length = response.headers().typed_get::<ContentLength>();
        if content_length.map(|v| v.0).unwrap_or(0) < 256 {
            return Ok(response);
        }

        // 3. 使用第一个支持的编码，将原始 body 流式压缩
        let encoding = match supported_encodings.first() {
            Some(encoding) => encoding,
            None => return Ok(response),
        };

        // 取出原始 body，放入空 body 占位
        let old_body = std::mem::replace(response.body_mut(), Body::empty());

        // 将原始 body 转为 frame stream
        let body_stream = body_to_stream(old_body);

        // 创建 duplex 管道：encoder 写入 writer 端，ReaderStream 读取 reader 端
        let (reader, writer) = tokio::io::duplex(8192);

        // 在后台 task 中执行流式压缩
        tokio::spawn(compress_stream(body_stream, writer, encoding.clone()));

        // 用 ReaderStream 将 reader 端转为 Bytes stream，再转为 Body
        let compressed_stream =
            ReaderStream::new(reader).map(|result| result.map_err(anyhow::Error::from));

        // 设置压缩响应头
        response.headers_mut().insert(
            CONTENT_ENCODING,
            encoding
                .as_header_value()
                .parse()
                .map_err(|err| ErrNo::Other(anyhow::Error::from(err)))?,
        );

        // 移除 Content-Length（流式压缩后长度不可预知）
        response.headers_mut().remove(hyper::header::CONTENT_LENGTH);

        // 设置压缩后的流式 body
        *response.body_mut() = Body::from_bytes_stream(compressed_stream);

        Ok(response)
    }
}

/// 后台 task：从 body stream 逐帧读取，写入 encoder，encoder 输出到 duplex writer
async fn compress_stream(
    mut body_stream: impl futures::Stream<Item = Result<Frame<Bytes>, anyhow::Error>> + Unpin,
    writer: tokio::io::DuplexStream,
    encoding: Encoding,
) {
    let result = compress_stream_inner(&mut body_stream, writer, encoding).await;
    if let Err(e) = result {
        log::error!("Response stream compression failed: {}", e);
    }
}

async fn compress_stream_inner(
    body_stream: &mut (impl futures::Stream<Item = Result<Frame<Bytes>, anyhow::Error>> + Unpin),
    mut writer: tokio::io::DuplexStream,
    encoding: Encoding,
) -> Result<(), anyhow::Error> {
    // 根据编码创建对应的 encoder，包裹 duplex writer
    match encoding {
        Encoding::Gzip => {
            let mut encoder = GzipEncoder::new(&mut writer);
            pump_stream_to_encoder(body_stream, &mut encoder).await?;
            encoder.shutdown().await?;
        }
        Encoding::Deflate => {
            let mut encoder = DeflateEncoder::new(&mut writer);
            pump_stream_to_encoder(body_stream, &mut encoder).await?;
            encoder.shutdown().await?;
        }
        Encoding::Brotli => {
            let mut encoder = BrotliEncoder::new(&mut writer);
            pump_stream_to_encoder(body_stream, &mut encoder).await?;
            encoder.shutdown().await?;
        }
        Encoding::Zstd => {
            let mut encoder = ZstdEncoder::new(&mut writer);
            pump_stream_to_encoder(body_stream, &mut encoder).await?;
            encoder.shutdown().await?;
        }
    }
    // writer 在此被 drop，reader 端会收到 EOF
    Ok(())
}

/// 从 body stream 中逐个 frame 读取数据，写入 encoder
async fn pump_stream_to_encoder(
    body_stream: &mut (impl futures::Stream<Item = Result<Frame<Bytes>, anyhow::Error>> + Unpin),
    encoder: &mut (impl AsyncWriteExt + Unpin),
) -> Result<(), anyhow::Error> {
    while let Some(frame) = body_stream.next().await {
        let frame: Frame<Bytes> = frame?;
        if let Some(data) = frame.data_ref() {
            encoder.write_all(data).await?;
        }
    }
    // 确保所有缓冲数据被刷入
    encoder.flush().await?;
    Ok(())
}

#[derive(Clone)]
pub struct CompressMiddleware;

impl<H, Ctx> Middleware<In<Ctx>, H> for CompressMiddleware
where
    H: Handler<In<Ctx>, Out = Out>,
    Ctx: Send + Sync + 'static,
{
    type Output = CompressHandler<H>;

    fn transform(self, handler: H) -> Self::Output {
        CompressHandler { inner: handler }
    }
}

impl CompressMiddleware {
    pub fn new() -> CompressMiddleware {
        CompressMiddleware
    }
}
