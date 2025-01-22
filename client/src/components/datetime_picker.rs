use crate::ArcFn;
use crate::Lazyable;
use crate::LightString;
use chrono::Datelike;
use chrono::Duration;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::Timelike;
use chrono::Weekday;
use gloo::timers::callback::Timeout;
use js_sys::Date;
use std::collections::HashMap;
use std::fmt;
use wasm_bindgen::JsCast;
use web_sys::EventTarget;
use web_sys::HtmlInputElement;
use yew::html::Scope;
use yew::prelude::*;
use yew::{html, Component, Context, Html};

struct State {
    value: Option<NaiveDateTime>,
    calendar: Calendar,
    tm_date: NaiveDate,
    tm_hours: u32,
    tm_minutes: u32,
    tm_seconds: u32,
    style: LightString,
}

pub enum Msg {
    PrevYear,
    NextYear,
    PrevMonth,
    NextMonth,
    SelectDate(NaiveDate),
    ChangeHours(String),
    ChangeMinutes(String),
    ChangeSeconds(String),
    MousewheelHours(WheelEvent),
    MousewheelMinutes(WheelEvent),
    MousewheelSeconds(WheelEvent),
    SetTargetSelected(EventTarget),
    Clear,
    Save,
    Noop,
}

#[derive(Clone, PartialEq)]
pub struct Config {
    first_day: Weekday,
    //客户端时间相对于服务端时间的偏移，单位为秒
    time_offset: i64,
    min_date: Option<Lazyable<NaiveDate>>,
    max_date: Option<Lazyable<NaiveDate>>,
    date_selectable: Option<ArcFn<NaiveDate, bool>>,
    check_date_time: Option<ArcFn<NaiveDateTime, Result<(), LightString>>>,
    decorate_date: Option<ArcFn<NaiveDate, Option<LightString>>>,
}

impl Config {
    fn new() -> Config {
        Default::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            first_day: Weekday::Sun,
            time_offset: 0,
            min_date: None,
            max_date: None,
            date_selectable: None,
            check_date_time: None,
            decorate_date: None,
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub value: Option<NaiveDateTime>,
    pub config: Option<Config>,
    pub ondone: Option<Callback<Option<NaiveDateTime>>>,
    #[prop_or(LightString::Static(""))]
    pub style: LightString,
}

pub struct DateTimePicker {
    config: Config,
    state: State,
    timeout: Option<Timeout>,
}

impl Component for DateTimePicker {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let config = props.config.clone().unwrap_or_else(|| Config::default());
        let initial_datetime = props
            .value
            .unwrap_or_else(|| get_curr_date_time() + Duration::seconds(config.time_offset));
        let initial_date = initial_datetime.date();
        let state = State {
            value: props.value,
            calendar: Calendar {
                solar_year: initial_date.year() as u32, //年
                solar_month: initial_date.month(),      //月
                first_day: config.first_day,            //周几排在最前面
            },
            tm_date: initial_date,
            tm_hours: initial_datetime.hour(),
            tm_minutes: initial_datetime.minute(),
            tm_seconds: initial_datetime.second(),
            style: props.style.clone(),
        };
        DateTimePicker {
            config: config,
            state,
            timeout: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PrevYear => {
                self.state.calendar.goto_prev_year();
            }
            Msg::NextYear => {
                self.state.calendar.goto_next_year();
            }
            Msg::PrevMonth => {
                self.state.calendar.goto_prev_month();
            }
            Msg::NextMonth => {
                self.state.calendar.goto_next_month();
            }
            Msg::SelectDate(date) => {
                self.select_date(date);
            }
            Msg::ChangeHours(hours) => {
                self.change_hours(&hours);
            }
            Msg::ChangeMinutes(minutes) => {
                self.change_minutes(&minutes);
            }
            Msg::ChangeSeconds(seconds) => {
                self.change_seconds(&seconds);
            }
            Msg::MousewheelHours(evt) => {
                self.handle_hours_mousewheel(ctx.link(), evt);
            }
            Msg::MousewheelMinutes(evt) => {
                self.handle_minutes_mousewheel(ctx.link(), evt);
            }
            Msg::MousewheelSeconds(evt) => {
                self.handle_seconds_mousewheel(ctx.link(), evt);
            }
            Msg::SetTargetSelected(target) => {
                self.timeout.take();
                set_target_selected(target);
            }
            Msg::Clear => {
                if let Some(ondone) = ctx.props().ondone.as_ref() {
                    ondone.emit(None);
                }
            }
            Msg::Save => {
                if let Some(ondone) = ctx.props().ondone.as_ref() {
                    if let Ok(date_time) = self.check_selected_date_time() {
                        ondone.emit(Some(date_time));
                    }
                }
            }
            Msg::Noop => (),
        }
        return true;
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        self.state.value = props.value;
        let config = props.config.clone().unwrap_or_else(|| Config::default());
        self.state.calendar.first_day = config.first_day;
        self.config = config;
        self.state.style = props.style.clone();
        return true;
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let on_prev_year_click = link.callback(|_evt: MouseEvent| Msg::PrevYear);
        let on_next_year_click = link.callback(|_evt: MouseEvent| Msg::NextYear);
        let on_prev_month_click = link.callback(|_evt: MouseEvent| Msg::PrevMonth);
        let on_next_month_click = link.callback(|_evt: MouseEvent| Msg::NextMonth);
        let on_change_hours = link.callback(|evt: InputEvent| {
            let input: HtmlInputElement = evt.target_unchecked_into();
            Msg::ChangeHours(input.value().trim().to_string())
        });
        let on_change_minutes = link.callback(|evt: InputEvent| {
            let input: HtmlInputElement = evt.target_unchecked_into();
            Msg::ChangeMinutes(input.value().trim().to_string())
        });
        let on_change_seconds = link.callback(|evt: InputEvent| {
            let input: HtmlInputElement = evt.target_unchecked_into();
            Msg::ChangeSeconds(input.value().trim().to_string())
        });
        let on_select_text = link.callback(|evt: FocusEvent| {
            if let Some(target) = evt.target() {
                set_target_selected(target);
            }
            Msg::Noop
        });
        let on_mousewheel_hours = link.callback(|evt: WheelEvent| Msg::MousewheelHours(evt));
        let on_mousewheel_minutes = link.callback(|evt: WheelEvent| Msg::MousewheelMinutes(evt));
        let on_mousewheel_seconds = link.callback(|evt: WheelEvent| Msg::MousewheelSeconds(evt));
        let on_clear = link.callback(|_evt: MouseEvent| Msg::Clear);
        let on_save = link.callback(|_evt: MouseEvent| Msg::Save);
        let weekdays = self.weekdays();
        let calendar_dates = self.calendar_dates();
        let selected_date_time_ret: Result<NaiveDateTime, LightString> =
            self.check_selected_date_time();
        let prev_year_btn_style = if self.prev_year_selectable() {
            "visibility:visible;"
        } else {
            "visibility:hidden;"
        };
        let next_year_btn_style = if self.next_year_selectable() {
            "visibility:visible;"
        } else {
            "visibility:hidden;"
        };
        let prev_month_btn_style = if self.prev_month_selectable() {
            "visibility:visible;"
        } else {
            "visibility:hidden;"
        };
        let next_month_btn_style = if self.next_month_selectable() {
            "visibility:visible;"
        } else {
            "visibility:hidden;"
        };
        html! {
            <div class="date-time-picker" style={self.state.style.clone()}>
                <table style="table-layout: fixed;border-collapse: collapse;">
                    <caption class="year-month-area">
                        <div class="year-area">
                            <span class="icon-prev" style={prev_year_btn_style} onclick={on_prev_year_click}>{"<"}</span>
                            <div class="year-wrapper">
                                <div style="width:100%;">
                                    {self.state.calendar.solar_year}{"年"}
                                </div>
                            </div>
                            <span class="icon-next" style={next_year_btn_style} onclick={on_next_year_click}>{">"}</span>
                        </div>
                        <div class="month-area">
                            <span class="icon-prev" style={prev_month_btn_style} onclick={on_prev_month_click}>{"<"}</span>
                            <div class="month-wrapper">
                                <div style="width:100%;">
                                    {self.state.calendar.solar_month}{"月"}
                                </div>
                            </div>
                            <span class="icon-next" style={next_month_btn_style} onclick={on_next_month_click}>{">"}</span>
                        </div>
                    </caption>
                    <thead>
                        <tr>
                            {
                                for weekdays.into_iter().map(|weekday| {
                                    html! {
                                        <th class="head-cell">
                                            {format_weekday(weekday)}
                                        </th>
                                    }
                                })
                            }
                        </tr>
                    </thead>
                    <tbody>
                        {
                            for calendar_dates.into_iter().map(|row| {
                                html! {
                                    <tr>
                                        {
                                            for row.into_iter().map(|cell| {
                                                let on_select_date = link.callback(move |_evt: MouseEvent| Msg::SelectDate(cell));
                                                html! {
                                                    <td class="cell" onclick={on_select_date} style={self.get_cell_style(cell)}>
                                                        {cell.day()}
                                                        {
                                                            if let Some(text) = self.decorate_date(cell) {
                                                                html! {
                                                                    <div class="date-decorator">{text}</div>
                                                                }
                                                            } else {
                                                                html! {}
                                                            }
                                                        }
                                                    </td>
                                                }
                                            })
                                        }
                                    </tr>
                                }
                            })
                        }
                    </tbody>
                </table>
                <div class="footer-area">
                    <div style="margin: 0.5em;overflow: auto;">
                        <div style="float:left;">
                            <span class="time-label" style="float: left;">{"时间"}</span>
                            <span style="display:inline-block;float: left;">
                                <input value={self.state.tm_hours.to_string()} oninput={on_change_hours} onfocus={on_select_text.clone()} onwheel={on_mousewheel_hours} class="text-box" type="text" style="border-right: none;" maxlength="2"/>
                                <span class="time-spliter">{":"}</span>
                                <input value={self.state.tm_minutes.to_string()} oninput={on_change_minutes} onfocus={on_select_text.clone()} onwheel={on_mousewheel_minutes} class="text-box" type="text" style="border-left: none;border-right: none;" maxlength="2"/>
                                <span class="time-spliter">{":"}</span>
                                <input value={self.state.tm_seconds.to_string()} oninput={on_change_seconds} onfocus={on_select_text} onwheel={on_mousewheel_seconds} class="text-box" type="text" style="border-left: none;" maxlength="2"/>
                            </span>
                        </div>
                        <div class="action-area" style="float: right;">
                            <button onclick={on_clear} class="btn-clear" style="margin-right: 0.5em;">{"清除"}</button>
                            <button onclick={on_save} disabled={selected_date_time_ret.is_err()} class="btn-save">{"确定"}</button>
                        </div>
                    </div>
                    {
                        match selected_date_time_ret {
                            Ok(date_time) => {
                                html! {
                                    <div style="text-align:center;">{format_date_time(date_time)}</div>
                                }
                            },
                            Err(err_msg) => {
                                html! {
                                    <div style="color:red;text-align:center;">{err_msg}</div>
                                }
                            }
                        }
                    }
                </div>
            </div>
        }
    }
}

impl DateTimePicker {
    fn weekdays(&self) -> Vec<Weekday> {
        let default = vec![
            Weekday::Sun,
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
        ];
        let mut index = 0;
        for (idx, item) in default.iter().enumerate() {
            if item == &self.state.calendar.first_day {
                index = idx;
                break;
            }
        }
        if 0 == index {
            return default;
        } else {
            return [&default[index..], &default[0..index]].concat();
        }
    }
    fn calendar_dates(&self) -> Vec<Vec<NaiveDate>> {
        self.state.calendar.get_grouped_dates()
    }
    fn get_cell_style(&self, date: NaiveDate) -> String {
        let mut style: HashMap<&'static str, &'static str> = HashMap::new();
        if !self.date_selectable(date) {
            //当天不可选，则置灰
            style.insert("background-color", "#eee");
            style.insert("color", "#aaa");
            style.insert("cursor", "not-allowed");
        } else {
            style.insert("cursor", "pointer");
            //是选中的那一天
            if self.state.tm_date == date {
                if let Some(date_time) = self.get_selected_date_time() {
                    if let Ok(()) = self.check_date_time(date_time) {
                        //时间格式正确，时间检查也通过
                        style.insert("background-color", "green");
                    } else {
                        //时间格式正确，但是时间检查不过
                        style.insert("background-color", "rgba(255, 0, 0, 0.3)");
                    }
                } else {
                    //时间格式不正确
                    style.insert("background-color", "green");
                }
                style.insert("color", "#fff");
            }
        }
        return Style(&style).to_string();
    }

    fn get_selected_date_time(&self) -> Option<NaiveDateTime> {
        if 24 <= self.state.tm_hours || 60 <= self.state.tm_minutes || 60 <= self.state.tm_seconds {
            return None;
        }
        let datetime = self.state.tm_date.and_hms(
            self.state.tm_hours,
            self.state.tm_minutes,
            self.state.tm_seconds,
        );
        return Some(datetime);
    }

    fn check_selected_date_time(&self) -> Result<NaiveDateTime, LightString> {
        if let Some(date_time) = self.get_selected_date_time() {
            if !self.date_selectable(date_time.date()) {
                return Err(LightString::Static("选中日期被禁用！"));
            } else {
                self.check_date_time(date_time)?;
                return Ok(date_time);
            }
        } else {
            return Err(LightString::Static("时间格式错误！"));
        }
    }

    fn check_date_time(&self, date_time: NaiveDateTime) -> Result<(), LightString> {
        if let Some(check_date_time) = self.config.check_date_time.as_ref() {
            return (check_date_time.0)(date_time);
        } else {
            return Ok(());
        }
    }

    fn get_min_date(&self) -> Option<NaiveDate> {
        return self.config.min_date.as_ref().map(|min_date| {
            return min_date.get().into_owned();
        });
    }

    fn get_max_date(&self) -> Option<NaiveDate> {
        return self.config.max_date.as_ref().map(|max_date| {
            return max_date.get().into_owned();
        });
    }

    //检查是否可以展示某个月的日历
    fn month_selectable(&self, year: u32, month: u32) -> bool {
        if let Some(min_date) = self.config.min_date.as_ref() {
            let min_date = min_date.get();
            if NaiveDate::from_ymd(year as i32, month, 1)
                < NaiveDate::from_ymd(min_date.year(), min_date.month(), 1)
            {
                return false;
            }
        }
        if let Some(max_date) = self.config.max_date.as_ref() {
            let max_date = max_date.get();
            if NaiveDate::from_ymd(year as i32, month, 1)
                > NaiveDate::from_ymd(max_date.year(), max_date.month(), 1)
            {
                return false;
            }
        }
        return true;
    }

    fn prev_year_selectable(&self) -> bool {
        let (solar_year, solar_month) = self.state.calendar.prev_year();
        return self.month_selectable(solar_year, solar_month);
    }

    fn next_year_selectable(&self) -> bool {
        let (solar_year, solar_month) = self.state.calendar.next_year();
        return self.month_selectable(solar_year, solar_month);
    }

    fn prev_month_selectable(&self) -> bool {
        let (solar_year, solar_month) = self.state.calendar.prev_month();
        return self.month_selectable(solar_year, solar_month);
    }

    fn next_month_selectable(&self) -> bool {
        let (solar_year, solar_month) = self.state.calendar.next_month();
        return self.month_selectable(solar_year, solar_month);
    }

    fn date_selectable(&self, date: NaiveDate) -> bool {
        let min_date = self.get_min_date();
        if let Some(min_date) = min_date {
            if date < min_date {
                return false;
            }
        }
        let max_date = self.get_max_date();
        if let Some(max_date) = max_date {
            if date > max_date {
                return false;
            }
        }
        if let Some(check_date) = self.config.date_selectable.as_ref() {
            if !(check_date.0)(date) {
                return false;
            }
        }
        return true;
    }

    fn select_date(&mut self, date: NaiveDate) {
        if !self.date_selectable(date) {
            //当天不能选择的，直接返回
            return;
        }
        self.state.tm_date = date;
    }

    fn change_hours(&mut self, hours: &str) {
        match u32::from_str_radix(&hours, 10) {
            Ok(hours) => {
                if 24 > hours {
                    self.state.tm_hours = hours;
                } else {
                    log::info!("小时超出正常值: {:?}", hours);
                }
            }
            Err(err) => {
                log::info!("小时格式不正确: {:?}", err);
            }
        }
    }

    fn change_minutes(&mut self, minutes: &str) {
        match u32::from_str_radix(&minutes, 10) {
            Ok(minutes) => {
                if 60 > minutes {
                    self.state.tm_minutes = minutes;
                } else {
                    log::info!("分钟超出正常值: {:?}", minutes);
                }
            }
            Err(err) => {
                log::info!("分钟格式不正确: {:?}", err);
            }
        }
    }
    fn change_seconds(&mut self, seconds: &str) {
        match u32::from_str_radix(&seconds, 10) {
            Ok(seconds) => {
                if 60 > seconds {
                    self.state.tm_seconds = seconds;
                } else {
                    log::info!("秒超出正常值: {:?}", seconds);
                }
            }
            Err(err) => {
                log::info!("秒格式不正确: {:?}", err);
            }
        }
    }
    fn handle_hours_mousewheel(&mut self, link: &Scope<Self>, evt: WheelEvent) {
        evt.prevent_default();
        let step = (evt.delta_y() / 100.0) as i32;
        let new_hours = self.state.tm_hours as i32 - step;
        self.state.tm_hours = new_hours.max(0).min(23) as u32;
        let link = link.clone();
        self.timeout = Some(Timeout::new(0, move || {
            let msg = if let Some(target) = evt.target() {
                Msg::SetTargetSelected(target)
            } else {
                Msg::Noop
            };
            link.send_message(msg)
        }));
    }
    fn handle_minutes_mousewheel(&mut self, link: &Scope<Self>, evt: WheelEvent) {
        evt.prevent_default();
        let step = (evt.delta_y() / 100.0) as i32;
        let new_minutes = self.state.tm_minutes as i32 - step;
        self.state.tm_minutes = new_minutes.max(0).min(59) as u32;
        let link = link.clone();
        self.timeout = Some(Timeout::new(0, move || {
            let msg = if let Some(target) = evt.target() {
                Msg::SetTargetSelected(target)
            } else {
                Msg::Noop
            };
            link.send_message(msg)
        }));
    }
    fn handle_seconds_mousewheel(&mut self, link: &Scope<Self>, evt: WheelEvent) {
        evt.prevent_default();
        let step = (evt.delta_y() / 100.0) as i32;
        let new_seconds = self.state.tm_seconds as i32 - step;
        self.state.tm_seconds = new_seconds.max(0).min(59) as u32;
        let link = link.clone();
        self.timeout = Some(Timeout::new(0, move || {
            let msg = if let Some(target) = evt.target() {
                Msg::SetTargetSelected(target)
            } else {
                Msg::Noop
            };
            link.send_message(msg)
        }));
    }

    fn decorate_date(&self, date: NaiveDate) -> Option<LightString> {
        return self
            .config
            .decorate_date
            .as_ref()
            .map(|decorate_date| (decorate_date.0)(date))
            .unwrap_or(None);
    }
}

fn set_target_selected(target: EventTarget) {
    match target.dyn_into::<HtmlInputElement>() {
        Ok(input_dom) => {
            let len = input_dom.value().len();
            set_text_selected(input_dom, 0, len as u32);
        }
        Err(err) => {
            log::error!("{:?}", err);
        }
    }
}

fn set_text_selected(input_dom: HtmlInputElement, start_index: u32, end_index: u32) {
    if let Err(err) = input_dom.set_selection_range(start_index, end_index) {
        log::error!("{:?}", err);
    }
    if let Err(err) = input_dom.focus() {
        log::error!("{:?}", err);
    }
}

struct Calendar {
    pub solar_year: u32,    //年
    pub solar_month: u32,   //月
    pub first_day: Weekday, //周几排在最前面
}

impl Calendar {
    fn prev_year(&self) -> (u32, u32) {
        let solar_year = self.solar_year - 1;
        if 1970 > solar_year {
            return (1970, 1);
        } else {
            return (solar_year, self.solar_month);
        }
    }
    fn next_year(&self) -> (u32, u32) {
        return (self.solar_year + 1, self.solar_month);
    }
    fn prev_month(&self) -> (u32, u32) {
        if 1 >= self.solar_month {
            let solar_year = self.solar_year - 1;
            if 1970 > solar_year {
                return (1970, 1);
            } else {
                return (solar_year, 12);
            }
        } else {
            return (self.solar_year, self.solar_month - 1);
        }
    }
    fn next_month(&self) -> (u32, u32) {
        if 12 <= self.solar_month {
            return (self.solar_year + 1, 1);
        } else {
            return (self.solar_year, self.solar_month + 1);
        }
    }
    fn goto_prev_month(&mut self) {
        let (solar_year, solar_month) = self.prev_month();
        self.solar_year = solar_year;
        self.solar_month = solar_month;
    }
    fn goto_next_month(&mut self) {
        let (solar_year, solar_month) = self.next_month();
        self.solar_year = solar_year;
        self.solar_month = solar_month;
    }
    fn goto_prev_year(&mut self) {
        let (solar_year, solar_month) = self.prev_year();
        self.solar_year = solar_year;
        self.solar_month = solar_month;
    }
    fn goto_next_year(&mut self) {
        let (solar_year, solar_month) = self.next_year();
        self.solar_year = solar_year;
        self.solar_month = solar_month;
    }
    fn get_continuous_dates(&self) -> Vec<NaiveDate> {
        let solar_year = self.solar_year;
        let solar_month = self.solar_month;
        let first_date_of_month = get_first_date_of_month(solar_year, solar_month);
        let pre_days_count = self.calc_pre_days_count();
        let first_display_date = first_date_of_month - Duration::days(pre_days_count as i64);
        let solar_days = get_solar_days(solar_year, solar_month);
        let next_days_count = self.calc_next_days_count();
        let len = pre_days_count + solar_days + next_days_count;
        return (0..len)
            .into_iter()
            .map(|index| {
                return first_display_date + Duration::days(index as i64);
            })
            .collect();
    }

    // 计算日历上需要显示的分好组之后的日期二维数组
    fn get_grouped_dates(&self) -> Vec<Vec<NaiveDate>> {
        return group_array(self.get_continuous_dates(), 7);
    }

    // 计算当月之前还需要显示几天
    fn calc_pre_days_count(&self) -> u8 {
        let first_date_of_month = get_first_date_of_month(self.solar_year, self.solar_month);
        let weekday = first_date_of_month.weekday();
        let mut pre_days_count: i8 = weekday as i8 - self.first_day as i8;
        if 0 > pre_days_count {
            pre_days_count += 7;
        }
        return pre_days_count as u8;
    }

    // 计算当月之后还需要显示几天
    fn calc_next_days_count(&self) -> u8 {
        let last_date_of_month = get_last_date_of_month(self.solar_year, self.solar_month);
        let weekday = last_date_of_month.weekday();
        let mut next_days_count: i8 = self.first_day as i8 - 1 - weekday as i8;
        if 0 > next_days_count {
            next_days_count = next_days_count + 7;
        }
        return next_days_count as u8;
    }
}

fn get_first_date_of_month(solar_year: u32, solar_month: u32) -> NaiveDate {
    return NaiveDate::from_ymd(solar_year as i32, solar_month, 1);
}

//返回1个月的最后1天（0点）
fn get_last_date_of_month(solar_year: u32, solar_month: u32) -> NaiveDate {
    let first_date_of_month = get_first_date_of_month(solar_year, solar_month);
    let solar_days = get_solar_days(solar_year, solar_month);
    return first_date_of_month + Duration::days(solar_days as i64 - 1);
}

const DAYS_OF_SOLAR_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

// 返回公历solar_year年solar_month月的天数
fn get_solar_days(solar_year: u32, solar_month: u32) -> u8 {
    if 2 == solar_month {
        if ((0 == solar_year % 4) && (0 != solar_year % 100)) || (0 == solar_year % 400) {
            return 29;
        } else {
            return 28;
        }
    } else {
        return DAYS_OF_SOLAR_MONTH[solar_month as usize - 1];
    }
}

fn group_array<T>(arr: Vec<T>, length: usize) -> Vec<Vec<T>> {
    let mut group: Vec<Vec<T>> = Vec::new();
    let mut group_item: Vec<T> = Vec::new();
    for item in arr {
        group_item.push(item);
        if length == group_item.len() {
            group.push(group_item);
            group_item = Vec::new();
        }
    }
    if !group_item.is_empty() {
        group.push(group_item);
    }
    return group;
}

fn format_date_time(datetime: NaiveDateTime) -> String {
    return format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        datetime.year(),
        datetime.month(),
        datetime.day(),
        datetime.hour(),
        datetime.minute(),
        datetime.second()
    );
}

fn format_weekday(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "一",
        Weekday::Tue => "二",
        Weekday::Wed => "三",
        Weekday::Thu => "四",
        Weekday::Fri => "五",
        Weekday::Sat => "六",
        Weekday::Sun => "日",
    }
}

fn get_curr_date_time() -> NaiveDateTime {
    let now = Date::new_0();
    return NaiveDate::from_ymd(
        now.get_full_year() as i32,
        now.get_month() + 1,
        now.get_date(),
    )
    .and_hms(now.get_hours(), now.get_minutes(), now.get_seconds());
}

struct Style<'a, K, V>(&'a HashMap<K, V>);

impl<K, V> fmt::Display for Style<'_, K, V>
where
    K: fmt::Display,
    V: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, value) in self.0 {
            write!(f, "{}:{};", key, value)?;
        }
        return Ok(());
    }
}

fn pkg_style<K, V>(style: &HashMap<K, V>) -> String
where
    K: fmt::Display,
    V: fmt::Display,
{
    return Style(style).to_string();
}
