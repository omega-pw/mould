use tihu::LightString;
use tokio_postgres::types::ToSql;

// fn calcDigitCount(n: usize) -> usize {
//     let mut last = n;
//     let mut count = 0;
//     loop {
//         count += 1;
//         last = last / 10;
//         if 0 == last {
//             break;
//         }
//     }
//     return count;
// }

const CONNECTOR: &str = " and ";

pub struct Condition {
    pub field: LightString,
    pub operator: Option<LightString>,
}

// impl Condition {
//     pub fn writeTo(&self, output: &mut String, pos: usize) {
//         output.push_str(self.field.borrow());
//         let strPos = pos.to_string();
//         match self.operator {
//             Some(ref operator) => {
//                 output.push_str(operator.borrow());
//                 output.push_str("$");
//                 output.push_str(&strPos);
//             },
//             None => {
//                 output.push_str("=$");
//                 output.push_str(&strPos);
//             }
//         }
//     }
//     pub fn len(&self, pos: usize) -> usize {
//         let posCount = calcDigitCount(pos);
//         let mut l = 0;
//         l += (self.field.borrow() as &str).len();
//         l += match self.operator {
//             Some(ref operator) => (operator.borrow() as &str).len() + "$".len() + posCount,
//             None => "=$".len() + posCount
//         };
//         return l;
//     }
// }

// pub fn getConditionsLen(conditions:&Vec<Condition>, startPos: usize) -> usize {
//     if conditions.is_empty() {
//         return 0;
//     }
//     let mut len = 0;
//     let mut idx = 0;
//     for condition in conditions {
//         len += condition.len(startPos + idx);
//         idx += 1;
//     }
//     len += (conditions.len() - 1) * CONNECTOR.len();
//     return len;
// }

// pub fn writeConditionsTo(conditions:&Vec<Condition>, output: &mut String, startPos: usize) {
//     let mut idx = 0;
//     for item in conditions {
//         if 0 != idx {
//             output.push_str(CONNECTOR);
//         }
//         item.writeTo(output, startPos + idx);
//         idx += 1;
//     }
// }

pub fn calc_sql_pagination(mut page_no: u64, mut page_size: u64) -> (u64, u64) {
    if 1 > page_no {
        page_no = 1;
    }
    if 1 > page_size {
        page_size = 1;
    }
    return (page_size, page_size * (page_no - 1));
}

pub fn add_val<'a, 'b>(
    vals: &'a mut Vec<&'b (dyn ToSql + std::marker::Sync)>,
    val: &'b (dyn ToSql + std::marker::Sync),
) -> String {
    vals.push(val);
    return format!("${}", vals.len());
}

pub fn add_vals<'a, 'b, T>(
    vals: &'a mut Vec<&'b (dyn ToSql + std::marker::Sync)>,
    val_list: &'b [T],
) -> String
where
    T: ToSql + std::marker::Sync,
{
    let mut index_list: Vec<String> = Vec::with_capacity(val_list.len());
    for val in val_list {
        vals.push(val);
        index_list.push(format!("${}", vals.len()));
    }
    return index_list.join(",");
}

pub fn add_conditions<'a, 'b, 'c>(
    vals: &'a mut Vec<&'c (dyn ToSql + std::marker::Sync)>,
    pairs: &'b [(Condition, &'c (dyn ToSql + std::marker::Sync))],
) -> String {
    return pairs
        .iter()
        .map(|(condition, val)| {
            [
                &condition.field,
                condition
                    .operator
                    .as_ref()
                    .map(|val| -> &str { &val })
                    .unwrap_or("="),
                &add_val(vals, *val),
            ]
            .concat()
        })
        .collect::<Vec<String>>()
        .join(CONNECTOR);
}
