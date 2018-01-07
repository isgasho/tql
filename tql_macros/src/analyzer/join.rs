/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

/// Analyzer for the join() method.

use syn::Expr;
use syn::spanned::Spanned;

use ast::{Expression, Join};
use error::{Error, Result, res};
use string::token_to_string;
use super::{check_field, mismatched_types, no_primary_key, path_expr_to_identifier};
use types::Type;

pub struct JoinData {
    join: Join,
}

/// Convert an `Expression` to a `Join`
pub fn argument_to_join(arg: &Expression, table_name: &str) -> Result<(Join, Vec<String>)> {
    let mut errors = vec![];
    let mut join = Join::default();
    let mut selected_fields = vec![];
    let mut related_table_name = String::new();

    if let Expr::Assign(ref assign) = *arg {
        if let Expr::Struct(ref structure) = *assign.right {
            related_table_name = token_to_string(&structure.path);
            for field in &structure.fields {
                selected_fields.push(format!("{}.{}", related_table_name, token_to_string(field)));
            }
        }
        else {
            return Err(vec![Error::new("Expecting structure expression, but got", assign.right.span())]); // TODO: improve error message.
        }

        if let Some(identifier) = path_expr_to_identifier(&assign.left, &mut errors) {
            let name = identifier.to_string();
            check_field(&identifier, arg.span(), &mut errors);
            // TODO: check that the field is a ForeignKey<_>.
            //mismatched_types("ForeignKey<_>", field_type, arg.span(), &mut errors);
            // TODO: check that the struct has the field id.
            // TODO: allow specifying an alternate primary key field.
            join = Join {
                base_field: name,
                base_table: table_name.to_string(),
                joined_field: "id".to_string(),
                joined_table: related_table_name,
            }
            // NOTE: if the field type is not an SQL table, an error is thrown by the
            // linter.
            // FIXME: update the previous comment since there's no more a linter.
        }
    }
    else {
        return Err(vec![Error::new("Expecting assignment, but got", arg.span())]); // TODO: improve error message.
    }
    res((join, selected_fields), errors)
}
