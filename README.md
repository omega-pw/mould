# mould
Mould是一个简单的任务编排工具，目标是通过任务和环境的解耦来减小各个环境之间的差异。希望它能像模具一样，制造出来的产品（环境）都是相同的，至少要差异极小。
这个项目还处在很早的阶段，是实验性的，请谨慎使用。



#### 使用方法

准备一个配置文件名字叫config.json5，demo配置如下：

```json
{
    host: "127.0.0.1",
    port: 8080,
    log_cfg_path: "./log4rs.yaml",
    extension_dir: "./extensions",
    job_log_dir: "./job_logs",
    sign_secret: "rX46ths0wP64ONdrDzIwAfnwWyBDJnGBkHqy1ri0UDiRwzcHXGT0yY01Rvox4LRKgYuID0Eppp9e6E8FSnuG16mld5Oek1nXvpBYlZtQQf62ACG4E6VrWSvJ4BPrnf522uIQ9OtUgPyiW2QrMnw8TaHidpK5yiHdV2QzxCcRvzlZBI2VVVEPTZ6GfQZyYSZc1idKQp1QMCT6suKJa9rX7iE8JV4Ayg7hlyQEPdElhAT6eqUUjQHGuG4Gt3XIBziE",
    rsa_pub_key: "./rsa-pub-key.pem",
    rsa_pri_key: "./rsa-pri-key.pem",
    server_random_value: "Q5rxHZPowd1Mc4eDczyo185R4XhO9RLPh1FGneWNBjW1",
    cache_server: {
        host: "127.0.0.1",
        port: 6379,
        user: null,
        password: null,
        max_size: 2,
    },
    data_source: {
        host: "127.0.0.1",
        port: 5432,
        dbname: "postgres",
        user: "postgres",
        password: "******",
        max_size: 2,
        ssl: null
    },
    public_path: "http://localhost:8080",
    oauth2_servers: {
        github: {
            auth_url: "https://github.com/login/oauth/authorize",
            token_url: "https://github.com/login/oauth/access_token",
            client_id: "xxx",
            client_secret: "xxx"
        }
    },
    openid_servers: {
        google: {
            name: "Google",
            issuer: "https://accounts.google.com",
            client_id: "xxxx.apps.googleusercontent.com",
            client_secret: "xxx",
            scopes: [
              "openid",
              "profile"
            ]
        }
    },
    "oss": {
        "access_key": "xxx",
        "secret_key": "xxx",
        "endpoint": "http://127.0.0.1:9000",
        "region": "us-east-1",
        "bucket": "mould"
    },
    email_account: {
        mail_host: "smtp.163.com",
        mail_port: 25,
        username: "testuser@163.com",
        password: "xxx",
        name: "testuser",
        address: "testuser@163.com"
    },
    email_template: {
        register_captcha: "./email_template/register_captcha.tmpl",
        reset_password_captcha: "./email_template/reset_password_captcha.tmpl"
    }
}
```
准备日志配置文件log4rs.yaml，参考log4rs库https://github.com/estk/log4rs
准备rsa公私钥对，参考https://travistidwell.com/jsencrypt/demo/index.html
准备redis服务配置、postgres数据库配置、兼容s3的对象存储配置、邮件账号配置，第三方账户认证系统（oauth2_servers或者openid_servers）配置可选
准备发送邮件的模板，注册邮件./email_template/register_captcha.tmpl，重置密码邮件./email_template/reset_password_captcha.tmpl，语法参考https://github.com/Keats/tera

准备扩展，扩展用于管理资源，环境可以认为是一系列资源的集合，目前提供了如下资源的简易扩展（etcd、kubernetes、mysql、nacos、postgresql、s3、server），请自行构建需要的扩展，把动态链接库放到extensions目录。

启动程序：
./mould ./config.json5

用浏览器访问 http://localhost:8080