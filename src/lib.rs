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
    use swc_core::ecma::transforms::typescript::strip;
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
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();
        chain!(
            resolver(unresolved_mark, top_level_mark, true),
            strip(top_level_mark),
            as_folder(super::EnumToObjVisitor)
        )
    }

    test!(SYNTAX, runner,
        /* Name */ normal_enum,
        /* Input */ r#"
            enum Foo {
                A,
                B
            }
        "#,
        /* Output */ r#"
            var Foo = {
                "A": 0,
                "0": "A",
                "B": 1,
                "1": "B",
            };
        "#
    );

    test!(SYNTAX, runner,
        /* Name */ exported_enum,
        /* Input */ r#"
            export enum Foo {
                A,
                B
            }
        "#,
        /* Output */ r#"
            export var Foo = {
                "A": 0,
                "0": "A",
                "B": 1,
                "1": "B",
            };
        "#
    );

    test!(SYNTAX, runner,
        /* Name */ const_enum,
        /* Input */ r#"
            const enum Foo {
                A = 0,
                B = 1,
                C = -1
                D = +2
            }
        "#,
        /* Output */ r#"
            var Foo = {
                "A": 0,
                0: "A",
                "B": 1,
                1: "B",
                "C": -1,
                "-1": "C",
                "D": 2,
                "2": "D",
            };
        "#
    );


    test!(SYNTAX, runner,
        /* Name */ exported_const_enum,
        /* Input */ r#"
            export const enum Foo {
                A,
                B
            }
        "#,
        /* Output */ r#"
            export var Foo = {
                "A": 0,
                "0": "A",
                "B": 1,
                "1": "B",
            };
        "#
    );

    test!(SYNTAX, runner,
        /* Name */ normal_compiled_enum,
        /* Input */ r#"
            var Foo;
            (function(Foo) {
                Foo[Foo["A"] = 0] = "A";
                Foo[Foo["B"] = 1] = "B";
            })(Foo || (Foo = {}));
        "#,
        /* Output */ r#"
            var Foo = {
                "A": 0,
                0: "A",
                "B": 1,
                1: "B",
            };
        "#
    );

    test!(SYNTAX, runner,
        /* Name */ exported_compiled_enum,
        /* Input */ r#"
            export var Foo;
            (function(Foo) {
                Foo[Foo["A"] = 0] = "A";
                Foo[Foo["B"] = 1] = "B";
            })(Foo || (Foo = {}));
        "#,
        /* Output */ r#"
            export var Foo = {
                "A": 0,
                0: "A",
                "B": 1,
                1: "B",
            };
        "#
    );
}
