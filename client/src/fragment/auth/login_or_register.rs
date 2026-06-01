use super::login::Login;
use super::register::Register;
use crate::assets;
use crate::components::button_group::ButtonGroup;
use crate::components::center_middle::CenterMiddle;
use crate::components::dialog::Dialog;
use crate::components::page::Page;
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
    tab: RwSignal<Tab>,
    ondone: UnsyncCallback<GetCurrUserResp>,
) -> impl IntoView {
    let navigate = use_navigate();
    let on_done = UnsyncCallback::new(move |curr_user| {
        ondone.run(curr_user);
        // handle_done(ctx, user);
    });
    view! {
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
                <ButtonGroup>
                    {
                        move || {
                            match tab.get() {
                                Tab::Login => {
                                    let navigate = navigate.clone();
                                    let on_reset_password = move |_| {
                                        navigate("/resetPassword", Default::default());
                                    };
                                    let on_switch_register = move |_| {
                                        tab.set(Tab::Register);
                                    };
                                    view! {
                                        <a href="javascript:void(0);" on:click={on_reset_password}>{"找回密码"}</a>
                                        <a href="javascript:void(0);" on:click={on_switch_register}>{"注册"}</a>
                                    }.into_any()
                                },
                                Tab::Register => {
                                    let on_switch_login = move |_| {
                                        tab.set(Tab::Login);
                                    };
                                    view! {
                                        <a href="javascript:void(0);" on:click={on_switch_login}>{"登录"}</a>
                                    }.into_any()
                                }
                            }
                        }
                    }
                </ButtonGroup>
            </div>
        </div>
    }
}

#[component]
pub fn LoginOrRegisterPage(
    #[prop(optional)] init_tab: Option<Tab>,
    ondone: UnsyncCallback<GetCurrUserResp>,
) -> impl IntoView {
    let tab: RwSignal<Tab> = RwSignal::new(init_tab.unwrap_or(Tab::Login));
    let title = {
        let tab = tab.clone();
        Signal::derive(move || match tab.get() {
            Tab::Login => SharedString::from("登录"),
            Tab::Register => SharedString::from("注册"),
        })
    };
    view! {
        <Page mask=false style={format!("background-repeat: no-repeat;background-size: cover;background-position: center;background-image:url({})", assets::LOGIN_BG.path())}>
            <CenterMiddle>
                <div style="text-align: center;">
                    <div style="display: flex;justify-content: center;align-items: center;">
                        <img src={assets::LOGO.path()} style="height: 4em;"/>
                        <span style="font-weight: bold;font-size:200%;margin-left: 0.25em;">{"Mould"}</span>
                    </div>
                    <p>{"减小环境之间的差异!"}</p>
                </div>
                <Dialog title={title.clone()} closable={false} content_style="background-color:#FFF;" style="margin-top:1em;">
                    <LoginOrRegister tab={tab} ondone={ondone}/>
                </Dialog>
            </CenterMiddle>
        </Page>
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
