mod enum_to_obj;

use enum_to_obj::EnumToObjVisitor;
use swc_core::ecma::ast::Program;
use swc_core::ecma::visit::{as_folder, FoldWith};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut as_folder(EnumToObjVisitor))
}

#[cfg(test)]
mod test {
    use swc_core::common::{chain, Mark};
    use swc_core::ecma::transforms::base::resolver;
    use swc_core::ecma::transforms::testing::Tester;
    use swc_core::ecma::{
        parser::{Syntax, TsConfig},
        transforms::testing::test,
        visit::{as_folder, Fold},
    };

    const SYNTAX: Syntax = Syntax::Typescript(TsConfig {
        tsx: true,
        decorators: false,
        dts: false,
        no_early_errors: false,
        disallow_ambiguous_jsx_like: true,
    });

    fn runner(_: &mut Tester) -> impl Fold {
        chain!(
            resolver(Mark::new(), Mark::new(), false),
            as_folder(super::EnumToObjVisitor)
        )
    }

    test!(SYNTAX, runner,
        /* Name */ bare_enum,
        /* Input */ r#"
            enum Foo {
                A,
                B
            }
        "#,
        /* Output */ r#"
            const Foo = {
                "A": 0,
                "B": 1
            };
        "#
    );

    test!(SYNTAX, runner,
        /* Name */ export_enum,
        /* Input */ r#"
            export enum Foo {
                A,
                B
            }
        "#,
        /* Output */ r#"
            export const Foo = {
                "A": 0,
                "B": 1
            };
        "#
    );
}
