pub enum Either<A, B> {
    Left(A),
    Right(B),
}

pub struct ListAction<Add, Update, Existed> {
    pub add_list: Vec<Add>,               //需要新增的记录
    pub update_list: Vec<Update>,         //需要更新的记录
    pub dropped_update_list: Vec<Update>, //需要丢弃的更新记录
    pub remove_list: Vec<Existed>,        //需要删除的记录
}

pub fn group_list_action<T, Add, Update, Existed>(
    save_list: Vec<T>,
    mut existed_list: Vec<Existed>,
    partition: impl Fn(T) -> Either<Add, Update>,
    equal: impl Fn(&Update, &Existed) -> bool,
    strict_equal: impl Fn(&Update, &Existed) -> bool,
    update_to_add: impl Fn(Update) -> Add,
) -> ListAction<Add, Update, Existed> {
    let mut add_list = Vec::new();
    let mut update_list = Vec::new();
    let mut dropped_update_list = Vec::new();
    for item in save_list {
        match partition(item) {
            Either::Left(add) => {
                add_list.push(add);
            }
            Either::Right(update) => {
                if let Some(existed) = existed_list
                    .iter()
                    .filter(|existed| equal(&update, existed))
                    .next()
                {
                    //找到了记录相同的
                    if strict_equal(&update, existed) {
                        //严格相同，并没有任何改动，则不进行更新
                        dropped_update_list.push(update);
                    } else {
                        //否则进行更新动作
                        update_list.push(update);
                    }
                } else {
                    //没有找到记录相同的已存在记录，按新增处理
                    add_list.push(update_to_add(update));
                }
            }
        }
    }
    //把不在更新列表里面的已存在记录移除
    existed_list.retain(|existed| {
        !update_list
            .iter()
            .chain(dropped_update_list.iter())
            .any(|update| equal(update, existed))
    });
    return ListAction {
        add_list: add_list,
        update_list: update_list,
        dropped_update_list: dropped_update_list,
        remove_list: existed_list,
    };
}

pub fn group_sub_list<T1, T2>(
    main_list: Vec<T1>,
    mut sub_list: Vec<T2>,
    equal: impl Fn(&T1, &T2) -> bool,
) -> Vec<(T1, Vec<T2>)> {
    let mut list = Vec::with_capacity(main_list.len());
    for main_data in main_list {
        let (mut current, rest): (Vec<_>, Vec<_>) = sub_list
            .into_iter()
            .partition(|sub_data| equal(&main_data, sub_data));
        list.push((main_data, current));
        sub_list = rest;
    }
    return list;
}
