use crate::SharedString;
use leptos::prelude::*;

#[component]
pub fn CenterMiddle(
    #[prop(into, optional)] content_style: SharedString,
    children: Children,
) -> impl IntoView {
    let mut style = String::from("display:inline-block;text-align: initial;");
    if !content_style.is_empty() {
        style.push_str(&content_style);
    }
    view! {
        <table style="width:100%;height: 100%;border-collapse: collapse;table-layout: auto;border: 0;">
            <tbody style="width:100%;height: 100%;">
                <tr style="width:100%;height: 100%;">
                    <td style="width:100%;height: 100%;vertical-align:middle;text-align: center;overflow: hidden;">
                        //text-align: initial 还原成默认的值
                        <div style={style}>
                            { children() }
                        </div>
                    </td>
                </tr>
            </tbody>
        </table>
    }
}
