use super::session::get_session_data;
use super::session::SessionId;
use super::session::SessionState;
use crate::Context;
use async_trait::async_trait;
use hyper::body::Incoming;
use hyper::Request;
use std::net::SocketAddr;
use std::sync::Arc;
use tihu::Id;
use tihu::SharedString;
use tihu_native::http::FromRequest;
use tihu_native::http::RequestData;
use tihu_native::ErrNo;

#[derive(Clone, Copy)]
pub struct OrphanedUser {
    pub session_id: SessionId,
    pub user_id: Id,
}

#[derive(Clone, Copy)]
pub struct User {
    pub session_id: SessionId,
    pub user_id: Id,
    pub org_id: Id,
}

#[derive(Clone, Copy)]
pub struct Guest {
    pub session_id: SessionId,
}

#[derive(Clone)]
pub enum AuthLevel {
    Guest(Guest),
    OrphanedUser(OrphanedUser),
    User(User),
}

impl AuthLevel {
    pub fn session_id(&self) -> SessionId {
        match self {
            AuthLevel::Guest(guest) => guest.session_id,
            AuthLevel::OrphanedUser(orphaned_user) => orphaned_user.session_id,
            AuthLevel::User(user) => user.session_id,
        }
    }

    pub fn guest(&self) -> Guest {
        match self {
            AuthLevel::Guest(guest) => guest.clone(),
            AuthLevel::OrphanedUser(orphaned_user) => Guest {
                session_id: orphaned_user.session_id,
            },
            AuthLevel::User(user) => Guest {
                session_id: user.session_id,
            },
        }
    }
}

#[async_trait]
impl FromRequest<Arc<Context>, ErrNo> for AuthLevel {
    async fn try_extract(
        context: &Arc<Context>,
        request: &Request<Incoming>,
        remote_addr: SocketAddr,
        request_data: &mut RequestData,
    ) -> Result<Self, ErrNo> {
        let session_state = request_data
            .try_get::<SessionState, Arc<Context>, ErrNo>(context, &request, remote_addr)
            .await?;
        let session_id = session_state.session.id();
        let session_info = get_session_data(context, &session_id.to_string()).await?;
        let auth_level = if let Some(session_info) = session_info {
            if let Some(org_id) = session_info.org_id {
                AuthLevel::User(User {
                    session_id: session_id,
                    user_id: session_info.user_id,
                    org_id: org_id,
                })
            } else {
                AuthLevel::OrphanedUser(OrphanedUser {
                    session_id: session_id,
                    user_id: session_info.user_id,
                })
            }
        } else {
            AuthLevel::Guest(Guest {
                session_id: session_id,
            })
        };
        return Ok(auth_level);
    }
}

#[async_trait]
impl FromRequest<Arc<Context>, ErrNo> for Guest {
    async fn try_extract(
        context: &Arc<Context>,
        request: &Request<Incoming>,
        remote_addr: SocketAddr,
        request_data: &mut RequestData,
    ) -> Result<Self, ErrNo> {
        let session_state = request_data
            .try_get::<SessionState, Arc<Context>, ErrNo>(context, &request, remote_addr)
            .await?;
        let session_id = session_state.session.id();
        return Ok(Guest {
            session_id: session_id,
        });
    }
}

#[async_trait]
impl FromRequest<Arc<Context>, ErrNo> for OrphanedUser {
    async fn try_extract(
        context: &Arc<Context>,
        request: &Request<Incoming>,
        remote_addr: SocketAddr,
        request_data: &mut RequestData,
    ) -> Result<Self, ErrNo> {
        let auth_level = request_data
            .try_get::<AuthLevel, Arc<Context>, ErrNo>(context, &request, remote_addr)
            .await?;
        match auth_level {
            AuthLevel::Guest(_) => {
                return Err(SharedString::from("用户未登录").into());
            }
            AuthLevel::OrphanedUser(orphaned_user) => {
                return Ok(orphaned_user.clone());
            }
            AuthLevel::User(user) => {
                return Ok(OrphanedUser {
                    session_id: user.session_id,
                    user_id: user.user_id,
                });
            }
        }
    }
}

#[async_trait]
impl FromRequest<Arc<Context>, ErrNo> for User {
    async fn try_extract(
        context: &Arc<Context>,
        request: &Request<Incoming>,
        remote_addr: SocketAddr,
        request_data: &mut RequestData,
    ) -> Result<Self, ErrNo> {
        let auth_level = request_data
            .try_get::<AuthLevel, Arc<Context>, ErrNo>(context, &request, remote_addr)
            .await?;
        match auth_level {
            AuthLevel::Guest(_) => {
                return Err(SharedString::from("用户未登录").into());
            }
            AuthLevel::OrphanedUser(_) => {
                return Err(SharedString::from("请先加入组织").into());
            }
            AuthLevel::User(user) => {
                return Ok(user.clone());
            }
        }
    }
}
