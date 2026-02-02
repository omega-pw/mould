use super::login::Login;
use super::register::Register;
use crate::components::modal_dialog::ModalDialog;
use crate::sdk;
use crate::SharedString;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use sdk::auth::get_curr_user::GetCurrUserResp;

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Login,
    Register,
}

#[component]
pub fn LoginOrRegister(
    #[prop(optional)] init_tab: Option<Tab>,
    ondone: UnsyncCallback<GetCurrUserResp>,
    #[prop(into, default = None)] oncancel: Option<UnsyncCallback<()>>,
) -> impl IntoView {
    let navigate = use_navigate();
    let tab: RwSignal<Tab> = RwSignal::new(init_tab.unwrap_or(Tab::Login));
    let on_switch_login = {
        let tab = tab.clone();
        move |_| {
            tab.set(Tab::Login);
        }
    };
    let on_switch_register = {
        let tab = tab.clone();
        move |_| {
            tab.set(Tab::Register);
        }
    };
    let on_done = UnsyncCallback::new(move |curr_user| {
        ondone.run(curr_user);
        // handle_done(ctx, user);
    });
    let title = {
        let tab = tab.clone();
        Signal::derive(move || match tab.get() {
            Tab::Login => SharedString::from("登录"),
            Tab::Register => SharedString::from("注册"),
        })
    };
    view! {
        <ModalDialog title={title} closable=true onclose={oncancel}>
            <div style="padding-top:2em;padding-bottom:2em;">
                <div style="padding-right:4em;">
                    {
                        let tab = tab.clone();
                        move || {
                            match tab.get() {
                                Tab::Login => {
                                    view! {
                                        <Login ondone={on_done} />
                                    }.into_any()
                                },
                                Tab::Register => {
                                    view! {
                                        <Register ondone={on_done} />
                                    }.into_any()
                                }
                            }
                        }
                    }
                </div>
                <div style="text-align:right;padding-right:1em;">
                    {
                        move || {
                            match tab.get() {
                                Tab::Login => {
                                    let navigate = navigate.clone();
                                    let on_reset_password = move |_| {
                                        navigate("/resetPassword", Default::default());
                                    };
                                    view! {
                                        <>
                                            <a href="javascript:void(0);" on:click={on_reset_password}>{"找回密码"}</a>
                                            <a href="javascript:void(0);" on:click={on_switch_register}>{"注册"}</a>
                                        </>
                                    }.into_any()
                                },
                                Tab::Register => {
                                    view! {
                                        <a href="javascript:void(0);" on:click={on_switch_login}>{"登录"}</a>
                                    }.into_any()
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
//     ctx.props().ondone.run(user);
// }

// pub fn login_or_register(
//     done_cb: UnsyncCallback<CurrOperator>,
//     cancel_cb: UnsyncCallback<()>,
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
//         ondone: UnsyncCallback::once(move |user: CurrUser| {
//             done_cb.run(user);
//             if let Some(inst_handle) = inst_handle_clone1.take() {
//                 inst_handle.destroy();
//                 let document = web_sys::window().unwrap().document().unwrap();
//                 let body = document.body().unwrap();
//                 body.remove_child(&inst_root_clone1).unwrap();
//             }
//         }),
//         oncancel: UnsyncCallback::once(move |_: ()| {
//             cancel_cb.run(());
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
