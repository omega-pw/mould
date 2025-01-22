use super::login::Login;
use super::register::Register;
use crate::components::modal_dialog::ModalDialog;
use crate::route::Route;
use crate::sdk;
use sdk::auth::get_curr_user::GetCurrUserResp;
use std::ops::Deref;
use yew::prelude::*;
use yew_router::components::Link;

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Login,
    Register,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub init_tab: Option<Tab>,
    pub ondone: Callback<GetCurrUserResp>,
    #[prop_or_default]
    pub oncancel: Callback<()>,
}

#[function_component]
pub fn LoginOrRegister(props: &Props) -> Html {
    let tab: UseStateHandle<Tab> = use_state(|| props.init_tab.unwrap_or(Tab::Login));
    let tab_clone = tab.clone();
    let on_switch_login = Callback::from(move |_| {
        tab_clone.set(Tab::Login);
    });
    let tab_clone = tab.clone();
    let on_switch_register = Callback::from(move |_| {
        tab_clone.set(Tab::Register);
    });
    let ondone = props.ondone.clone();
    let on_done = Callback::from(move |curr_user| {
        ondone.emit(curr_user);
        // handle_done(ctx, user);
    });
    let title = match tab.deref() {
        Tab::Login => "登录",
        Tab::Register => "注册",
    };
    html! {
        <ModalDialog title={title} closable=true onclose={props.oncancel.clone()}>
            <div style="padding-top:2em;padding-bottom:2em;">
                <div style="padding-right:4em;">
                    {
                        match tab.deref() {
                            Tab::Login => {
                                html! {
                                    <Login ondone={on_done.clone()} />
                                }
                            },
                            Tab::Register => {
                                html! {
                                    <Register ondone={on_done} />
                                }
                            }
                        }
                    }
                </div>
                <div style="text-align:right;padding-right:1em;">
                    {
                        match tab.deref() {
                            Tab::Login => {
                                html! {
                                    <>
                                        <Link<Route> to={Route::ResetPassword}>{"找回密码"}</Link<Route>>
                                        <a href="javascript:void(0);" onclick={on_switch_register}>{"注册"}</a>
                                    </>
                                }
                            },
                            Tab::Register => {
                                html! {
                                    <a href="javascript:void(0);" onclick={on_switch_login}>{"登录"}</a>
                                }
                            }
                        }
                    }
                </div>
            </div>
        </ModalDialog>
    }
}

// fn handle_done(&mut self, ctx: &Context<Self>, user: CurrUser) {
//     let curr_user = CurrUserOpt {
//         user_id: user.user_id,
//         user_name: user.user_name.clone(),
//     };
//     unsafe {
//         let old_ctx = crate::CONTEXT.clone();
//         crate::CONTEXT.curr_user = Some(Rc::new(curr_user));
//         let new_ctx = crate::CONTEXT.clone();
//         event::emit(event::ContextChange, (new_ctx, old_ctx));
//     }
//     ctx.props().ondone.emit(user);
// }

// pub fn login_or_register(
//     done_cb: Callback<CurrOperator>,
//     cancel_cb: Callback<()>,
//     init_tab: Option<Tab>,
// ) {
//     let document = web_sys::window().unwrap().document().unwrap();
//     let body = document.body().unwrap();
//     let inst_root = document.create_element("div").unwrap();
//     body.append_child(&inst_root).unwrap();
//     let inst_handle: Rc<Cell<Option<AppHandle<LoginOrRegister>>>> = Rc::new(Cell::new(None));
//     let inst_handle_clone1 = inst_handle.clone();
//     let inst_handle_clone2 = inst_handle.clone();
//     let inst_root_clone1 = inst_root.clone();
//     let inst_root_clone2 = inst_root.clone();
//     let props = Props {
//         init_tab: init_tab,
//         ondone: Callback::once(move |user: CurrUser| {
//             done_cb.emit(user);
//             if let Some(inst_handle) = inst_handle_clone1.take() {
//                 inst_handle.destroy();
//                 let document = web_sys::window().unwrap().document().unwrap();
//                 let body = document.body().unwrap();
//                 body.remove_child(&inst_root_clone1).unwrap();
//             }
//         }),
//         oncancel: Callback::once(move |_: ()| {
//             cancel_cb.emit(());
//             if let Some(inst_handle) = inst_handle_clone2.take() {
//                 inst_handle.destroy();
//                 let document = web_sys::window().unwrap().document().unwrap();
//                 let body = document.body().unwrap();
//                 body.remove_child(&inst_root_clone2).unwrap();
//             }
//         }),
//     };
//     inst_handle.set(Some(
//         yew::start_app_with_props_in_element::<LoginOrRegister>(inst_root, props),
//     ));
// }
