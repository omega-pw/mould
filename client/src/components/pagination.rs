use yew::prelude::*;
use yew::{html, Component, Context, Html};

struct State {
    pagination: tihu::Pagination,
}

pub enum Msg {
    Page(u64),
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub pagination: tihu::Pagination,
    pub onpage: Callback<u64>,
}

pub struct Pagination {
    state: State,
}

impl Component for Pagination {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let state = State {
            pagination: props.pagination.clone(),
        };
        Pagination { state }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Page(page) => {
                ctx.props().onpage.emit(page);
            }
        }
        return true;
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        self.state.pagination = props.pagination.clone();
        return true;
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let page_no = self.state.pagination.page_no;
        let page_count = self.state.pagination.page_count;
        let link = ctx.link();
        let on_first_page = link.callback(|_| Msg::Page(1));
        let on_pre_page = link.callback(move |_| Msg::Page(page_no - 1));
        let on_last_page = link.callback(move |_| Msg::Page(page_count.max(1)));
        let on_next_page = link.callback(move |_| Msg::Page(page_no + 1));
        html! {
            <div class="pagination" style="display:inline-block;">
                <button onclick={on_first_page.clone()}>{"首页"}</button>
                <button disabled={!self.state.pagination.has_pre_page} class={self.get_pre_page_class()} onclick={on_pre_page}><span></span></button>
                {
                    if self.calc_start_at_first() {
                        html! {}
                    } else {
                        html! {
                            <button class={self.get_page_class(1)} onclick={on_first_page}>{1}</button>
                        }
                    }
                }
                {
                    if !self.calc_start_at_first() && self.calc_has_start_clearance() {
                        html! {
                            <span>{"..."}</span>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    for self.calc_middle_page_list().into_iter().map(|page| {
                        let on_middle_page = link.callback(move |_| Msg::Page(page));
                        html! {
                            <button class={self.get_page_class(page)} onclick={on_middle_page}>{page}</button>
                        }
                    })
                }
                {
                    if !self.calc_end_at_last() && self.calc_has_end_clearance() {
                        html! {
                            <span>{"..."}</span>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    if self.calc_end_at_last() {
                        html! {}
                    } else {
                        html! {
                            <button class={self.get_page_class(self.state.pagination.page_count)} onclick={on_last_page.clone()}>{self.state.pagination.page_count}</button>
                        }
                    }
                }
                <button disabled={!self.state.pagination.has_next_page} class={self.get_next_page_class()} onclick={on_next_page}><span></span></button>
                <button onclick={on_last_page}>{"尾页"}</button>
            </div>
        }
    }
}

impl Pagination {
    fn calc_middle_page_list(&self) -> Vec<u64> {
        let mut page_list: Vec<u64> = Vec::new();
        for page in self.state.pagination.start_page.max(1)..(self.state.pagination.end_page + 1) {
            page_list.push(page);
        }
        return page_list;
    }

    fn calc_start_at_first(&self) -> bool {
        return 1 == self.state.pagination.start_page;
    }

    fn calc_end_at_last(&self) -> bool {
        return self.state.pagination.page_count == self.state.pagination.end_page;
    }

    fn calc_has_start_clearance(&self) -> bool {
        return 2 < self.state.pagination.start_page;
    }

    fn calc_has_end_clearance(&self) -> bool {
        return self.state.pagination.end_page < self.state.pagination.page_count - 1;
    }

    fn get_page_class(&self, page_no: u64) -> &'static str {
        if page_no == self.state.pagination.page_no {
            return "num-btn active";
        } else {
            return "num-btn";
        }
    }

    fn get_pre_page_class(&self) -> &'static str {
        if self.state.pagination.has_pre_page {
            return "point-left";
        } else {
            return "point-left invalid";
        }
    }

    fn get_next_page_class(&self) -> &'static str {
        if self.state.pagination.has_next_page {
            return "point-right";
        } else {
            return "point-right invalid";
        }
    }
}
