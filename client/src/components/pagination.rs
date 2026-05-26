use leptos::prelude::*;

#[component]
pub fn Pagination(
    #[prop(into)] pagination: Signal<tihu::Pagination>,
    onpage: UnsyncCallback<u64>,
) -> impl IntoView {
    let on_first_page = {
        let onpage = onpage.clone();
        move |_| {
            onpage.run(1);
        }
    };
    let on_pre_page = {
        let pagination = pagination.clone();
        let onpage = onpage.clone();
        move |_| {
            let page_no = pagination.read().page_no;
            onpage.run(page_no - 1);
        }
    };
    let on_last_page = {
        let pagination = pagination.clone();
        let onpage = onpage.clone();
        move |_| {
            let page_count = pagination.read().page_count;
            onpage.run(page_count.max(1));
        }
    };
    let on_next_page = {
        let pagination = pagination.clone();
        let onpage = onpage.clone();
        move |_| {
            let page_no = pagination.read().page_no;
            onpage.run(page_no + 1);
        }
    };
    view! {
        <div class="pagination" style="display:inline-block;">
            <button on:click={on_first_page.clone()}>{"首页"}</button>
            <button disabled={
                let pagination = pagination.clone();
                move || {
                    !pagination.read().has_pre_page
                }
            } class={
                let pagination = pagination.clone();
                move || {
                    get_pre_page_class(&pagination.read())
                }
            } on:click={on_pre_page}><span></span></button>
            <Show
                when={
                    let pagination = pagination.clone();
                    move || { !calc_start_at_first(&pagination.read()) }
                }
            >
                <button class={
                    let pagination = pagination.clone();
                    move || {
                        get_page_class(&pagination.read(), 1)
                    }
                } on:click={on_first_page}>{1}</button>
            </Show>
            <Show
                when={
                    let pagination = pagination.clone();
                    move || { !calc_start_at_first(&pagination.read()) && calc_has_start_clearance(&pagination.read()) }
                }
            >
                <span>{"..."}</span>
            </Show>
            <For
                each={
                    let pagination = pagination.clone();
                    move || { calc_middle_page_list(&pagination.read()).into_iter() }
                }
                key=|page| { *page }
                children={
                    let pagination = pagination.clone();
                    move |page| {
                        let on_middle_page = move |_| {
                            onpage.run(page);
                        };
                        view! {
                            <button class={
                                let pagination = pagination.clone();
                                move || {
                                    get_page_class(&pagination.read(), page)
                                }
                            } on:click={on_middle_page}>{page}</button>
                        }
                    }
                }
            />
            <Show
                when={
                    let pagination = pagination.clone();
                    move || { !calc_end_at_last(&pagination.read()) && calc_has_end_clearance(&pagination.read()) }
                }
            >
                <span>{"..."}</span>
            </Show>
            <Show
                when={
                    let pagination = pagination.clone();
                    move || { !calc_end_at_last(&pagination.read()) }
                }
            >
                <button class={
                    let pagination = pagination.clone();
                    move || {
                        get_page_class(&pagination.read(), pagination.read().page_count)
                    }
                } on:click={on_last_page.clone()}>{pagination.read().page_count}</button>
            </Show>
            <button disabled={
                let pagination = pagination.clone();
                move || {
                    !pagination.read().has_next_page
                }
            } class={
                let pagination = pagination.clone();
                move || {
                    get_next_page_class(&pagination.read())
                }
            } on:click={on_next_page}><span></span></button>
            <button on:click={on_last_page}>{"尾页"}</button>
        </div>
    }
}

fn calc_middle_page_list(pagination: &tihu::Pagination) -> Vec<u64> {
    let mut page_list: Vec<u64> = Vec::new();
    for page in pagination.start_page.max(1)..(pagination.end_page + 1) {
        page_list.push(page);
    }
    return page_list;
}

fn calc_start_at_first(pagination: &tihu::Pagination) -> bool {
    return 1 == pagination.start_page;
}

fn calc_end_at_last(pagination: &tihu::Pagination) -> bool {
    return pagination.page_count == pagination.end_page;
}

fn calc_has_start_clearance(pagination: &tihu::Pagination) -> bool {
    return 2 < pagination.start_page;
}

fn calc_has_end_clearance(pagination: &tihu::Pagination) -> bool {
    return pagination.end_page < pagination.page_count - 1;
}

fn get_page_class(pagination: &tihu::Pagination, page_no: u64) -> &'static str {
    if page_no == pagination.page_no {
        return "num-btn active";
    } else {
        return "num-btn";
    }
}

fn get_pre_page_class(pagination: &tihu::Pagination) -> &'static str {
    if pagination.has_pre_page {
        return "point-left";
    } else {
        return "point-left invalid";
    }
}

fn get_next_page_class(pagination: &tihu::Pagination) -> &'static str {
    if pagination.has_next_page {
        return "point-right";
    } else {
        return "point-right invalid";
    }
}
