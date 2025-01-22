use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
pub struct Bytes {
    inst: JsValue,
}

impl Bytes {
    fn get_js_class() -> js_sys::Function {
        js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("Bytes"))
            .unwrap()
            .dyn_into()
            .unwrap()
    }
    pub fn new(bytes: &[u8]) -> Self {
        let js_class = Self::get_js_class();
        let bytes = Uint8Array::from(bytes);
        let args = js_sys::Array::new();
        args.push(&bytes);
        let inst = js_sys::Reflect::construct(&js_class, &args).unwrap();
        Self { inst }
    }
    pub fn get_raw(&self) -> Vec<u8> {
        let get_raw_method: js_sys::Function =
            js_sys::Reflect::get(&self.inst, &JsValue::from_str("getRaw"))
                .unwrap()
                .dyn_into()
                .unwrap();
        let bytes: Uint8Array = get_raw_method
            .call0(&self.inst)
            .unwrap()
            .dyn_into()
            .unwrap();
        return bytes.to_vec();
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct AesKey256 {
    inst: JsValue,
}

impl AesKey256 {
    fn get_js_class() -> js_sys::Function {
        js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("AesKey256"))
            .unwrap()
            .dyn_into()
            .unwrap()
    }
    pub fn from_bytes(key: &[u8; 32]) -> Self {
        let js_class = Self::get_js_class();
        let bytes = Bytes::new(key);
        let args = js_sys::Array::new();
        args.push(&bytes.inst);
        let inst = js_sys::Reflect::construct(&js_class, &args).unwrap();
        Self { inst }
    }
    pub fn new_random() -> Self {
        let js_class = Self::get_js_class();
        let new_random_method: js_sys::Function =
            js_sys::Reflect::get(&js_class, &JsValue::from_str("newRandom"))
                .unwrap()
                .dyn_into()
                .unwrap();
        let inst = new_random_method
            .call0(&wasm_bindgen::JsValue::UNDEFINED)
            .unwrap();
        Self { inst }
    }
    pub fn get_bytes(&self) -> Vec<u8> {
        let get_bytes_method: js_sys::Function =
            js_sys::Reflect::get(&self.inst, &JsValue::from_str("getBytes"))
                .unwrap()
                .dyn_into()
                .unwrap();
        let bytes = get_bytes_method.call0(&self.inst).unwrap();
        return (Bytes { inst: bytes }).get_raw();
    }
    pub fn encrypt(&self, plain: &[u8]) -> Vec<u8> {
        let encrypt_method: js_sys::Function =
            js_sys::Reflect::get(&self.inst, &JsValue::from_str("encrypt"))
                .unwrap()
                .dyn_into()
                .unwrap();
        let plain = Bytes::new(plain);
        let cipher = encrypt_method.call1(&self.inst, &plain.inst).unwrap();
        return (Bytes { inst: cipher }).get_raw();
    }
    pub fn decrypt(&self, cipher: &[u8]) -> Vec<u8> {
        let decrypt_method: js_sys::Function =
            js_sys::Reflect::get(&self.inst, &JsValue::from_str("decrypt"))
                .unwrap()
                .dyn_into()
                .unwrap();
        let cipher = Bytes::new(cipher);
        let plain = decrypt_method.call1(&self.inst, &cipher.inst).unwrap();
        return (Bytes { inst: plain }).get_raw();
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct RsaPubKey2048 {
    inst: JsValue,
}

impl RsaPubKey2048 {
    fn get_js_class() -> js_sys::Function {
        js_sys::Reflect::get(
            &web_sys::window().unwrap(),
            &JsValue::from_str("RsaPubKey2048"),
        )
        .unwrap()
        .dyn_into()
        .unwrap()
    }
    pub fn try_from_string(pub_key: &str) -> Self {
        let js_class = Self::get_js_class();
        let args = js_sys::Array::new();
        args.push(&JsValue::from_str(pub_key));
        let inst = js_sys::Reflect::construct(&js_class, &args).unwrap();
        Self { inst }
    }
    pub fn get_string(&self) -> String {
        let get_string_method: js_sys::Function =
            js_sys::Reflect::get(&self.inst, &JsValue::from_str("getString"))
                .unwrap()
                .dyn_into()
                .unwrap();
        let pub_key = get_string_method.call0(&self.inst).unwrap();
        return pub_key.as_string().unwrap();
    }
    pub fn encrypt(&self, plain: &[u8]) -> Option<Vec<u8>> {
        let encrypt_method: js_sys::Function =
            js_sys::Reflect::get(&self.inst, &JsValue::from_str("encrypt"))
                .unwrap()
                .dyn_into()
                .unwrap();
        let plain = Bytes::new(plain);
        let cipher = encrypt_method.call1(&self.inst, &plain.inst).unwrap();
        if cipher.is_null() || cipher.is_undefined() {
            return None;
        } else {
            return Some((Bytes { inst: cipher }).get_raw());
        }
    }
    pub fn verify(&self, plain: &[u8], signature: &[u8]) -> Option<bool> {
        let verify_method: js_sys::Function =
            js_sys::Reflect::get(&self.inst, &JsValue::from_str("verify"))
                .unwrap()
                .dyn_into()
                .unwrap();
        let plain = Bytes::new(plain);
        let signature = Bytes::new(signature);
        let result = verify_method
            .call2(&self.inst, &plain.inst, &signature.inst)
            .unwrap();
        return result.as_bool();
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct RsaPriKey2048 {
    inst: JsValue,
}

impl RsaPriKey2048 {
    fn get_js_class() -> js_sys::Function {
        js_sys::Reflect::get(
            &web_sys::window().unwrap(),
            &JsValue::from_str("RsaPriKey2048"),
        )
        .unwrap()
        .dyn_into()
        .unwrap()
    }
    pub fn try_from_string(pri_key: &str) -> Self {
        let js_class = Self::get_js_class();
        let args = js_sys::Array::new();
        args.push(&JsValue::from_str(pri_key));
        let inst = js_sys::Reflect::construct(&js_class, &args).unwrap();
        Self { inst }
    }
    pub fn get_string(&self) -> String {
        let get_string_method: js_sys::Function =
            js_sys::Reflect::get(&self.inst, &JsValue::from_str("getString"))
                .unwrap()
                .dyn_into()
                .unwrap();
        let pri_key = get_string_method.call0(&self.inst).unwrap();
        return pri_key.as_string().unwrap();
    }
    pub fn decrypt(&self, cipher: &[u8]) -> Option<Vec<u8>> {
        let decrypt_method: js_sys::Function =
            js_sys::Reflect::get(&self.inst, &JsValue::from_str("decrypt"))
                .unwrap()
                .dyn_into()
                .unwrap();
        let cipher = Bytes::new(cipher);
        let plain = decrypt_method.call1(&self.inst, &cipher.inst).unwrap();
        if plain.is_null() || plain.is_undefined() {
            return None;
        } else {
            return Some((Bytes { inst: plain }).get_raw());
        }
    }
    pub fn sign(&self, plain: &[u8]) -> Option<Vec<u8>> {
        let sign_method: js_sys::Function =
            js_sys::Reflect::get(&self.inst, &JsValue::from_str("sign"))
                .unwrap()
                .dyn_into()
                .unwrap();
        let plain = Bytes::new(plain);
        let signature = sign_method.call1(&self.inst, &plain.inst).unwrap();
        if signature.is_null() || signature.is_undefined() {
            return None;
        } else {
            return Some((Bytes { inst: signature }).get_raw());
        }
    }
}

pub fn gen_rsa_key_pair() -> (RsaPubKey2048, RsaPriKey2048) {
    let js_function: js_sys::Function = js_sys::Reflect::get(
        &web_sys::window().unwrap(),
        &JsValue::from_str("genRsaKeyPair"),
    )
    .unwrap()
    .dyn_into()
    .unwrap();
    let key_pair = js_function
        .call0(&wasm_bindgen::JsValue::UNDEFINED)
        .unwrap();
    let public = js_sys::Reflect::get(&key_pair, &JsValue::from_str("public")).unwrap();
    let private = js_sys::Reflect::get(&key_pair, &JsValue::from_str("private")).unwrap();
    return (
        RsaPubKey2048 { inst: public },
        RsaPriKey2048 { inst: private },
    );
}

pub fn is_valid_rsa_key_pair(pub_key: &RsaPubKey2048, pri_key: &RsaPriKey2048) -> bool {
    let js_function: js_sys::Function = js_sys::Reflect::get(
        &web_sys::window().unwrap(),
        &JsValue::from_str("isValidRsaKeyPair"),
    )
    .unwrap()
    .dyn_into()
    .unwrap();
    let result = js_function
        .call2(
            &wasm_bindgen::JsValue::UNDEFINED,
            &pub_key.inst,
            &pri_key.inst,
        )
        .unwrap();
    return result.as_bool().unwrap();
}

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let data = Bytes::new(data);
    let sha256_method: js_sys::Function =
        js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("sha256"))
            .unwrap()
            .dyn_into()
            .unwrap();
    let hash = sha256_method
        .call1(&wasm_bindgen::JsValue::UNDEFINED, &data.inst)
        .unwrap();
    let hash_bytes = (Bytes { inst: hash }).get_raw();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hash_bytes);
    return hash;
}

pub fn sha512(data: &[u8]) -> [u8; 64] {
    let data = Bytes::new(data);
    let sha512_method: js_sys::Function =
        js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("sha512"))
            .unwrap()
            .dyn_into()
            .unwrap();
    let hash = sha512_method
        .call1(&wasm_bindgen::JsValue::UNDEFINED, &data.inst)
        .unwrap();
    let hash_bytes = (Bytes { inst: hash }).get_raw();
    let mut hash = [0u8; 64];
    hash.copy_from_slice(&hash_bytes);
    return hash;
}
