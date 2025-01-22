import CryptoJS from 'crypto-js';
import { BigInteger } from 'jsencrypt/lib/lib/jsbn/jsbn';
import { SecureRandom } from 'jsencrypt/lib/lib/jsbn/rng';
import { JSEncryptRSAKey } from "jsencrypt/lib/JSEncryptRSAKey";
import JSEncrypt from 'jsencrypt';

function calcWord(num: number[]): number {
    return num[0] * 256 * 256 * 256 + num[1] * 256 * 256 + num[2] * 256 + num[3];
}

function uint8ArrayToWordArray(bytes: Uint8Array): CryptoJS.lib.WordArray {
    let num = [0, 0, 0, 0];
    let result: number[] = [];
    for (let i = 0; i < bytes.length; i++) {
        let mod = i % 4;
        num[mod] = bytes[i];
        if (3 == mod) {
            result.push(calcWord(num));
            num = [0, 0, 0, 0];
        }
    }
    if (0 != bytes.length % 4) {
        result.push(calcWord(num));
    }
    return CryptoJS.lib.WordArray.create(result, bytes.length);
}

function wordArrayToUint8Array(wordArray: CryptoJS.lib.WordArray): Uint8Array {
    var sigBytes = wordArray.sigBytes;
    let bytes = new Uint8Array(sigBytes);
    for (var i = 0; i < sigBytes; i++) {
        var bite = (wordArray.words[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xff;
        bytes[i] = bite;
    }
    return bytes;
}

export class Bytes {
    private bytes: CryptoJS.lib.WordArray;
    constructor(bytes: Uint8Array) {
        this.bytes = uint8ArrayToWordArray(bytes);
        //如果不对bytes进行禁止改造动作，当放Bytes到vue的data或者store里面的时候，会导致新执行的CryptoJS.enc.Base64.parse报栈溢出错误
        Object.defineProperty(this, "bytes", {
            configurable: false
        });
    }
    static newRandom(count: number): Bytes {
        let bytes = CryptoJS.lib.WordArray.random(count);
        return Bytes.__fromWordArray(bytes);
    }
    getRaw(): Uint8Array {
        return wordArrayToUint8Array(this.bytes);
    }
    __getWordArray(): CryptoJS.lib.WordArray {
        return this.bytes;
    }
    static __fromWordArray(bytes: CryptoJS.lib.WordArray): Bytes {
        let inst = new Bytes(new Uint8Array());
        inst.bytes = bytes;
        return inst;
    }
    length(): number {
        return this.bytes.sigBytes;
    }
    concat(other: Bytes): Bytes {
        return Bytes.__fromWordArray(this.bytes.concat(other.bytes));
    }
    toHex(): string {
        return this.bytes.toString(CryptoJS.enc.Hex);
    }
    toBase64(): string {
        return this.bytes.toString(CryptoJS.enc.Base64);
    }
    static tryFromHex(data: string): Bytes {
        let bytes = CryptoJS.enc.Hex.parse(data);
        return Bytes.__fromWordArray(bytes);
    }
    static tryFromBase64(data: string): Bytes {
        let bytes = CryptoJS.enc.Base64.parse(data);
        return Bytes.__fromWordArray(bytes);
    }
    static fromUtf8(data: string): Bytes {
        let encoder = new TextEncoder();
        return new Bytes(encoder.encode(data));
    }
}

export class AesKey256 {
    private key: Bytes;
    private constructor(key: Bytes) {
        this.key = key;
    }
    static tryFromBytes(key: Bytes): AesKey256 {
        if (32 !== key.length()) {
            throw new Error("Aes key must be 32 bytes.");
        }
        return new AesKey256(key);
    }
    static newRandom(): AesKey256 {
        let key = Bytes.newRandom(32);
        return new AesKey256(key);
    }
    getBytes(): Bytes {
        return this.key;
    }
    encrypt(plain: Bytes): Bytes {
        const cipher = CryptoJS.AES.encrypt(
            plain.__getWordArray(),
            this.key.__getWordArray(),
            {
                mode: CryptoJS.mode.ECB,
                padding: CryptoJS.pad.Pkcs7,
            }
        ).ciphertext;
        return Bytes.__fromWordArray(cipher);
    }
    decrypt(cipher: Bytes): Bytes {
        const plain = CryptoJS.AES.decrypt(
            CryptoJS.lib.CipherParams.create({ ciphertext: cipher.__getWordArray() }),
            this.key.__getWordArray(),
            {
                mode: CryptoJS.mode.ECB,
                padding: CryptoJS.pad.Pkcs7,
            }
        );
        return Bytes.__fromWordArray(plain);
    }
}

export class RsaPubKey2048 {
    private pubKey: string;
    private enc: JSEncrypt;
    private constructor(pubKey: string) {
        this.pubKey = pubKey;
        var enc = new JSEncrypt({
            default_key_size: 2048 as any
        });
        enc.setPublicKey(pubKey);
        this.enc = enc;
    }
    static tryFromString(pubKey: string): RsaPubKey2048 {
        return new RsaPubKey2048(pubKey);
    }
    getString(): string {
        return this.pubKey;
    }
    encrypt(plain: Bytes): Bytes | null {
        let cipher = publicEncrypt(this.enc.getKey(), plain.getRaw());
        return cipher ? new Bytes(cipher) : null;
    }
    verify(plain: Bytes, signature: Bytes): boolean | null {
        return verifySha256(this.enc.getKey(), plain.getRaw(), signature.getRaw());
    }
}

export class RsaPriKey2048 {
    private priKey: string;
    private enc: JSEncrypt;
    private constructor(priKey: string) {
        this.priKey = priKey;
        var enc = new JSEncrypt({
            default_key_size: 2048 as any
        });
        enc.setPrivateKey(priKey);
        this.enc = enc;
    }
    static tryFromString(priKey: string): RsaPriKey2048 {
        return new RsaPriKey2048(priKey);
    }
    getString(): string {
        return this.priKey;
    }
    decrypt(cipher: Bytes): Bytes | null {
        let plain = privateDecrypt(this.enc.getKey(), cipher.getRaw());
        return plain ? new Bytes(plain) : null;
    }
    sign(plain: Bytes): Bytes | null {
        let signature = signSha256(this.enc.getKey(), plain.getRaw());
        return signature ? new Bytes(signature) : null;
    }
}

function pkcs1pad1(s: string, n: number) {
    if (n < s.length + 22) {
        console.error("Message too long for RSA");
        return null;
    }
    var len = n - s.length - 6;
    var filler = "";
    for (var f = 0; f < len; f += 2) {
        filler += "ff";
    }
    var m = "0001" + filler + "00" + s;
    return new BigInteger(m, 16);
}

// PKCS#1 (type 2, random) pad input string s to n bytes, and return a bigint
function pkcs1pad2(s: Uint8Array, n: number): BigInteger | null {
    if (n < s.length + 11) { // TODO: fix for utf-8
        console.error("Message too long for RSA");
        return null;
    }
    var ba: number[] = [];
    var i = s.length - 1;
    while (i >= 0 && n > 0) {
        ba[--n] = s[i--];
    }
    ba[--n] = 0;
    var rng = new SecureRandom();
    var x: number[] = [];
    while (n > 2) { // random non-zero pad
        x[0] = 0;
        while (x[0] == 0) {
            rng.nextBytes(x);
        }
        ba[--n] = x[0];
    }
    ba[--n] = 2;
    ba[--n] = 0;
    return new BigInteger(ba);
}

// Undo PKCS#1 (type 2, random) padding and, if valid, return the plaintext
function pkcs1unpad2(d: BigInteger, n: number): Uint8Array | null {
    var b = d.toByteArray();
    var i = 0;
    while (i < b.length && b[i] == 0) {
        ++i;
    }
    if (b.length - i != n - 1 || b[i] != 2) {
        return null;
    }
    ++i;
    while (b[i] != 0) {
        if (++i >= b.length) {
            return null;
        }
    }
    var ret: number[] = [];
    while (++i < b.length) {
        var c = b[i] & 255;
        ret.push(c);
    }
    return Uint8Array.from(ret);
}

function publicEncrypt(pubKey: JSEncryptRSAKey, bytes: Uint8Array): Uint8Array | null {
    var maxLength = ((pubKey as any).n.bitLength() + 7) >> 3;
    var m = pkcs1pad2(bytes, maxLength);
    if (m == null) {
        return null;
    }
    var c = pubKey.doPublic(m);
    if (c == null) {
        return null;
    }
    var h = c.toString(16);
    var length = h.length;
    // fix zero before result
    for (var i = 0; i < maxLength * 2 - length; i++) {
        h = "0" + h;
    }
    return wordArrayToUint8Array(CryptoJS.enc.Hex.parse(h));
}

function privateDecrypt(priKey: JSEncryptRSAKey, bytes: Uint8Array): Uint8Array | null {
    var c = new BigInteger(uint8ArrayToWordArray(bytes).toString(CryptoJS.enc.Hex), 16);
    var m = priKey.doPrivate(c);
    if (m == null) {
        return null;
    }
    return pkcs1unpad2(m, ((priKey as any).n.bitLength() + 7) >> 3);
}

// https://tools.ietf.org/html/rfc3447#page-43
var DIGEST_HEADERS: {
    [key: string]: string;
} = {
    md2: "3020300c06082a864886f70d020205000410",
    md5: "3020300c06082a864886f70d020505000410",
    sha1: "3021300906052b0e03021a05000414",
    sha224: "302d300d06096086480165030402040500041c",
    sha256: "3031300d060960864801650304020105000420",
    sha384: "3041300d060960864801650304020205000430",
    sha512: "3051300d060960864801650304020305000440",
    ripemd160: "3021300906052b2403020105000414"
};

function getDigestHeader(name: string) {
    return DIGEST_HEADERS[name] || "";
}

function removeDigestHeader(str: string) {
    for (var name_1 in DIGEST_HEADERS) {
        if (DIGEST_HEADERS.hasOwnProperty(name_1)) {
            var header = DIGEST_HEADERS[name_1];
            var len = header.length;
            if (str.substr(0, len) == header) {
                return str.substr(len);
            }
        }
    }
    return str;
}

function signSha256(priKey: JSEncryptRSAKey, bytes: Uint8Array) {
    var header = getDigestHeader("sha256");
    var digest = header + CryptoJS.SHA256(uint8ArrayToWordArray(bytes)).toString(CryptoJS.enc.Hex);
    var m = pkcs1pad1(digest, (priKey as any).n.bitLength() / 4);
    if (m == null) {
        return null;
    }
    var c = priKey.doPrivate(m);
    if (c == null) {
        return null;
    }
    var h = c.toString(16);
    var length = h.length;
    var maxLength = (((priKey as any) as any).n.bitLength() + 7) >> 3;
    // fix zero before result
    for (var i = 0; i < maxLength * 2 - length; i++) {
        h = "0" + h;
    }
    return wordArrayToUint8Array(CryptoJS.enc.Hex.parse(h));
}

function verifySha256(pubKey: JSEncryptRSAKey, bytes: Uint8Array, signature: Uint8Array) {
    var c = new BigInteger(uint8ArrayToWordArray(signature).toString(CryptoJS.enc.Hex), 16);
    var m = pubKey.doPublic(c);
    if (m == null) {
        return null;
    }
    var unpadded = m.toString(16).replace(/^1f+00/, "");
    var digest = removeDigestHeader(unpadded);
    return digest == CryptoJS.SHA256(uint8ArrayToWordArray(bytes)).toString(CryptoJS.enc.Hex);
}

export function genRsaKeyPair() {
    var encrypt = new JSEncrypt({
        default_key_size: 2048 as any
    });
    return {
        public: RsaPubKey2048.tryFromString(encrypt.getPublicKey()),
        private: RsaPriKey2048.tryFromString(encrypt.getPrivateKey()),
    };
}

export function isValidRsaKeyPair(rsaPubKey: RsaPubKey2048, rsaPriKey: RsaPriKey2048) {
    let data = Bytes.newRandom(1024);
    let signature = rsaPriKey.sign(data);
    if (!signature) {
        return false;
    }
    let ok = rsaPubKey.verify(data, signature);
    return !!ok;
}

export function sha256(data: Bytes): Bytes {
    const hash = CryptoJS.SHA256(data.__getWordArray());
    return Bytes.__fromWordArray(hash);
}

export function sha512(data: Bytes): Bytes {
    const hash = CryptoJS.SHA512(data.__getWordArray());
    return Bytes.__fromWordArray(hash);
}

export async function calcFileSha512(file: File): Promise<Bytes> {
    const stream = file.stream();
    const reader = stream.getReader();
    let done = false;
    const sha512 = CryptoJS.algo.SHA512.create();
    while (!done) {
        const ret = await reader.read();
        done = ret.done;
        const chunk = ret.value;
        if (!done) {
            sha512.update(uint8ArrayToWordArray(chunk!));
        }
    }
    return Bytes.__fromWordArray(sha512.finalize());
}

export async function deriveKeyByPBKDF2(password: Bytes, salt: Bytes, count: number): Promise<Bytes> {
    let key = CryptoJS.PBKDF2(password.__getWordArray(), salt.__getWordArray(), { keySize: count / 4, iterations: 100000, hasher: CryptoJS.algo.SHA256 });
    return Bytes.__fromWordArray(key);
}
