use js_sys::Date;

/**
 * 把10进制数字转换成62位进制字符串
 * @param {u64} number 10进制数字
 * @return {String} 62位进制字符串
 */
fn num_to_radix62(number: u64) -> String {
    let chars = b"0123456789abcdefghigklmnopqrstuvwxyzABCDEFGHIGKLMNOPQRSTUVWXYZ";
    let radix = chars.len() as u64;
    let mut qutient = number;
    let mut arr: Vec<u8> = Vec::new();
    loop {
        let rest = qutient % radix;
        qutient = (qutient - rest) / radix;
        arr.push(chars[rest as usize]);
        if 0 == qutient {
            break;
        }
    }
    arr.reverse();
    return String::from_utf8_lossy(&arr).into();
}

pub struct IdGen {
    prefix: Option<String>,
    last_time: u64,
    last_increase: u64,
}

impl IdGen {
    pub fn new(prefix: Option<String>) -> IdGen {
        let curr_time = Date::now().abs() as u64;
        IdGen {
            prefix: prefix,
            last_time: curr_time,
            last_increase: 0,
        }
    }

    /**
     * id生成方法
     * @return {String} 返回字符串id
     */
    pub fn generate(&mut self) -> String {
        let curr_time = Date::now().abs() as u64;
        let prefix: String = self
            .prefix
            .as_ref()
            .map(|prefix| prefix.clone())
            .unwrap_or_else(|| String::from(""));
        if curr_time == self.last_time {
            self.last_increase += 1;
            return prefix + &num_to_radix62(curr_time) + &num_to_radix62(self.last_increase);
        } else {
            self.last_time = curr_time;
            self.last_increase = 0;
            return prefix + &num_to_radix62(curr_time);
        }
    }
}
