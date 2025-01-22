//运行时publicPath在最前面设置
if("undefined" !== typeof window && window["STATIC_PATH"]){
    __webpack_public_path__ = window["STATIC_PATH"];
    if(!/\/$/.test(__webpack_public_path__)){
        //保证__webpack_public_path__以斜杠结尾，不然STATIC_PATH如果路径没有提供斜杠，webpack加载分片代码路径不正确
        __webpack_public_path__ = __webpack_public_path__ + "/";
    }
}
require("./bootstrap.js").default();