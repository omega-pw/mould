import "./common";

let pkg = import("./pkg");

export default function () {
    pkg.then(module => {
        module.run_app();
    });
}