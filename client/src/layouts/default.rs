use super::header_bar::HeaderBar;
use super::sys_menu::SysMenu;
use crate::assets;
use crate::components::center_middle::CenterMiddle;
use crate::components::container_layout::Aside;
use crate::components::container_layout::Header;
use crate::components::container_layout::HorizontalLayout;
use crate::components::container_layout::Main;
use crate::components::container_layout::VerticalLayout;
use crate::sdk;
use leptos::prelude::*;
use leptos_router::nested_router::Outlet;
use sdk::auth::get_curr_user::User;

#[component]
pub fn DefaultLayout(
    curr_user: RwSignal<Option<User>>,
    onexit: UnsyncCallback<()>,
) -> impl IntoView {
    move || {
        if let Some(curr_user) = curr_user.get() {
            if curr_user.org_id.is_some() {
                view! {
                    <HorizontalLayout class="height-fill">
                        <Aside class="border-box height-fill" style="width:16em;border-right: 1px solid #CCC;">
                            <VerticalLayout>
                                <Header style="padding:0.25em;text-align:center;border-bottom: 1px solid #CCC;">
                                    <div style="display: flex;justify-content: center;align-items: center;">
                                        <img src={assets::LOGO.path()} style="height: 3em;"/>
                                        <span style="font-weight: bold;font-size:150%;margin-left: 0.25em;">{"Mould"}</span>
                                    </div>
                                </Header>
                                <Main style="overflow-y: auto;">
                                    <SysMenu permissions={Vec::new()}/>
                                </Main>
                            </VerticalLayout>
                        </Aside>
                        <Main class="height-fill">
                            <VerticalLayout>
                                <Header style="padding:0.25em;border-bottom: 1px solid #CCC;">
                                    <HeaderBar curr_user={curr_user} onexit={onexit}></HeaderBar>
                                </Header>
                                <Main style="overflow-y: auto;">
                                    <Outlet/>
                                </Main>
                            </VerticalLayout>
                        </Main>
                    </HorizontalLayout>
                }.into_any()
            } else {
                view! {
                    <CenterMiddle>
                        {format!("你还没有加入任何组织，请联系组织成员添加，你的id：{}", curr_user.id)}
                    </CenterMiddle>
                }.into_any()
            }
        } else {
            view! {}.into_any()
        }
    }
}
