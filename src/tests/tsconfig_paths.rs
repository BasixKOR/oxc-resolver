//! tests for tsconfig-paths
//!
//! Fixtures copied from <https://github.com/parcel-bundler/parcel/tree/v2/packages/utils/node-resolver-core/test/fixture/tsconfig>.

use std::path::{Path, PathBuf};

use crate::{
    JSONError, ResolveError, ResolveOptions, Resolver, TsConfig, TsconfigOptions,
    TsconfigReferences,
};

// <https://github.com/parcel-bundler/parcel/blob/b6224fd519f95e68d8b93ba90376fd94c8b76e69/packages/utils/node-resolver-rs/src/lib.rs#L2303>
#[test]
fn tsconfig_resolve() {
    let f = super::fixture_root().join("tsconfig");

    #[rustfmt::skip]
    let pass = [
        (f.clone(), None, "ts-path", f.join("src/foo.js")),
        (f.join("nested"), None, "ts-path", f.join("nested/test.js")),
        (f.join("cases/index"), None, "foo", f.join("node_modules/tsconfig-index/foo.js")),
        // This requires reading package.json.tsconfig field
        // (f.join("cases/field"), "foo", f.join("node_modules/tsconfig-field/foo.js"))
        (f.join("cases/exports"), None, "foo", f.join("node_modules/tsconfig-exports/foo.js")),
        (f.join("cases/extends-extension"), None, "foo", f.join("cases/extends-extension/foo.js")),
        (f.join("cases/extends-extensionless"), None, "foo", f.join("node_modules/tsconfig-field/foo.js")),
        (f.join("cases/extends-paths"), Some("src"), "@/index", f.join("cases/extends-paths/src/index.js")),
        (f.join("cases/extends-multiple"), None, "foo", f.join("cases/extends-multiple/foo.js")),
        (f.join("cases/absolute-alias"), None, "/images/foo.js", f.join("cases/absolute-alias/public/images/foo.ts")),
    ];

    for (dir, subdir, request, expected) in pass {
        let resolver = Resolver::new(ResolveOptions {
            tsconfig: Some(TsconfigOptions {
                config_file: dir.join("tsconfig.json"),
                references: TsconfigReferences::Auto,
            }),
            extension_alias: vec![(".js".into(), vec![".js".into(), ".ts".into(), ".tsx".into()])],
            ..ResolveOptions::default()
        });
        let path = subdir.map_or(dir.clone(), |subdir| dir.join(subdir));
        let resolved_path = resolver.resolve(&path, request).map(|f| f.full_path());
        assert_eq!(resolved_path, Ok(expected), "{request} {path:?}");
    }

    #[rustfmt::skip]
    let data = [
        (f.join("node_modules/tsconfig-not-used"), "ts-path", Ok(f.join("src/foo.js"))),
    ];

    let resolver = Resolver::new(ResolveOptions {
        tsconfig: Some(TsconfigOptions {
            config_file: f.join("tsconfig.json"),
            references: TsconfigReferences::Auto,
        }),
        ..ResolveOptions::default()
    });
    for (path, request, expected) in data {
        let resolution = resolver.resolve(&path, request).map(|f| f.full_path());
        assert_eq!(resolution, expected, "{path:?} {request}");
    }
}

#[test]
fn tsconfig_fallthrough() {
    let f = super::fixture_root().join("tsconfig");

    let resolver = Resolver::new(ResolveOptions {
        tsconfig: Some(TsconfigOptions {
            config_file: f.join("tsconfig.json"),
            references: TsconfigReferences::Auto,
        }),
        ..ResolveOptions::default()
    });

    let resolved_path = resolver.resolve(&f, "/");
    assert_eq!(resolved_path, Err(ResolveError::NotFound("/".into())));
}

#[test]
fn json_with_comments() {
    let f = super::fixture_root().join("tsconfig/cases/trailing-comma");

    let resolver = Resolver::new(ResolveOptions {
        tsconfig: Some(TsconfigOptions {
            config_file: f.join("tsconfig.json"),
            references: TsconfigReferences::Auto,
        }),
        ..ResolveOptions::default()
    });

    let resolved_path = resolver.resolve(&f, "foo").map(|f| f.full_path());
    assert_eq!(resolved_path, Ok(f.join("bar.js")));
}

#[test]
fn with_bom() {
    let f = super::fixture_root().join("tsconfig/cases/with-bom");

    let resolver = Resolver::new(ResolveOptions {
        tsconfig: Some(TsconfigOptions {
            config_file: f.join("tsconfig.json"),
            references: TsconfigReferences::Auto,
        }),
        ..ResolveOptions::default()
    });

    let resolved_path = resolver.resolve(&f, "foo").map(|f| f.full_path());
    assert_eq!(resolved_path, Ok(f.join("bar.js")));
}

#[test]
fn broken() {
    let f = super::fixture_root().join("tsconfig");

    let resolver = Resolver::new(ResolveOptions {
        tsconfig: Some(TsconfigOptions {
            config_file: f.join("tsconfig_broken.json"),
            references: TsconfigReferences::Auto,
        }),
        ..ResolveOptions::default()
    });

    let resolved_path = resolver.resolve(&f, "/");
    let error = ResolveError::Json(JSONError {
        path: f.join("tsconfig_broken.json"),
        message: String::from("EOF while parsing an object at line 2 column 0"),
        line: 2,
        column: 0,
    });
    assert_eq!(resolved_path, Err(error));
}

#[test]
fn empty() {
    let f = super::fixture_root().join("tsconfig/cases/empty");

    let resolver = Resolver::new(ResolveOptions {
        tsconfig: Some(TsconfigOptions {
            config_file: f.join("tsconfig.json"),
            references: TsconfigReferences::Auto,
        }),
        ..ResolveOptions::default()
    });

    let resolved_path = resolver.resolve(&f, "./index").map(|f| f.full_path());
    assert_eq!(resolved_path, Ok(f.join("index.js")));
}

// <https://github.com/parcel-bundler/parcel/blob/c8f5c97a01f643b4d5c333c02d019ef2618b44a5/packages/utils/node-resolver-rs/src/tsconfig.rs#L193C12-L193C12>
#[test]
fn test_paths() {
    let path = Path::new("/foo/tsconfig.json");
    let mut tsconfig_json = serde_json::json!({
        "compilerOptions": {
            "paths": {
                "jquery": ["node_modules/jquery/dist/jquery"],
                "*": ["generated/*"],
                "bar/*": ["test/*"],
                "bar/baz/*": ["baz/*", "yo/*"],
                "@/components/*": ["components/*"],
                "url": ["node_modules/my-url"],
            }
        }
    })
    .to_string();
    let tsconfig = TsConfig::parse(true, path, &mut tsconfig_json).unwrap().build();

    let data = [
        ("jquery", vec!["/foo/node_modules/jquery/dist/jquery"]),
        ("test", vec!["/foo/generated/test"]),
        ("test/hello", vec!["/foo/generated/test/hello"]),
        ("bar/hi", vec!["/foo/test/hi"]),
        ("bar/baz/hi", vec!["/foo/baz/hi", "/foo/yo/hi"]),
        ("@/components/button", vec!["/foo/components/button"]),
        ("./jquery", vec![]),
        ("url", vec!["/foo/node_modules/my-url"]),
    ];

    for (specifier, expected) in data {
        let paths = tsconfig.resolve_path_alias(specifier);
        let expected = expected.into_iter().map(PathBuf::from).collect::<Vec<_>>();
        assert_eq!(paths, expected, "{specifier}");
    }
}

// <https://github.com/parcel-bundler/parcel/blob/c8f5c97a01f643b4d5c333c02d019ef2618b44a5/packages/utils/node-resolver-rs/src/tsconfig.rs#L233C6-L233C19>
#[test]
fn test_base_url() {
    let path = Path::new("/foo/tsconfig.json");
    let mut tsconfig_json = serde_json::json!({
        "compilerOptions": {
            "baseUrl": "./src"
        }
    })
    .to_string();
    let tsconfig = TsConfig::parse(true, path, &mut tsconfig_json).unwrap().build();

    let data = [
        ("foo", vec!["/foo/src/foo"]),
        ("components/button", vec!["/foo/src/components/button"]),
        ("./jquery", vec![]),
    ];

    for (specifier, expected) in data {
        let paths = tsconfig.resolve_path_alias(specifier);
        let expected = expected.into_iter().map(PathBuf::from).collect::<Vec<_>>();
        assert_eq!(paths, expected, "{specifier}");
    }
}

// <https://github.com/parcel-bundler/parcel/blob/c8f5c97a01f643b4d5c333c02d019ef2618b44a5/packages/utils/node-resolver-rs/src/tsconfig.rs#L252>
#[test]
fn test_paths_and_base_url() {
    let path = Path::new("/foo/tsconfig.json");
    let mut tsconfig_json = serde_json::json!({
        "compilerOptions": {
            "baseUrl": "./src",
            "paths": {
                "*": ["generated/*"],
                "bar/*": ["test/*"],
                "bar/baz/*": ["baz/*", "yo/*"],
                "@/components/*": ["components/*"]
            }
        }
    })
    .to_string();
    let tsconfig = TsConfig::parse(true, path, &mut tsconfig_json).unwrap().build();

    let data = [
        ("test", vec!["/foo/src/generated/test", "/foo/src/test"]),
        ("test/hello", vec!["/foo/src/generated/test/hello", "/foo/src/test/hello"]),
        ("bar/hi", vec!["/foo/src/test/hi", "/foo/src/bar/hi"]),
        ("bar/baz/hi", vec!["/foo/src/baz/hi", "/foo/src/yo/hi", "/foo/src/bar/baz/hi"]),
        ("@/components/button", vec!["/foo/src/components/button", "/foo/src/@/components/button"]),
        ("./jquery", vec![]),
    ];

    for (specifier, expected) in data {
        let paths = tsconfig.resolve_path_alias(specifier);
        let expected = expected.into_iter().map(PathBuf::from).collect::<Vec<_>>();
        assert_eq!(paths, expected, "{specifier}");
    }
}

#[test]
fn test_merge_tsconfig() {
    let resolver = Resolver::default();
    let dir = super::fixture_root().join("tsconfig/cases/merge_compiler_options");
    let resolution = resolver.resolve_tsconfig(&dir).expect("resolved");
    let compiler_options = resolution.compiler_options();
    assert_eq!(compiler_options.experimental_decorators, Some(true));
    assert_eq!(compiler_options.jsx, Some("react-jsx".to_string()));
    assert_eq!(compiler_options.jsx_factory, Some("h".to_string()));
    assert_eq!(compiler_options.jsx_fragment_factory, Some("Fragment".to_string()));
    assert_eq!(compiler_options.jsx_import_source, Some("xxx".to_string()));
    assert_eq!(compiler_options.module, Some("ESNext".to_string()));
    assert_eq!(compiler_options.target, Some("ESNext".to_string()));
}

#[test]
fn test_no_merge_tsconfig() {
    let resolver = Resolver::default();
    let dir = super::fixture_root().join("tsconfig/cases/no_merge_compiler_options");
    let resolution = resolver.resolve_tsconfig(&dir).expect("resolved");
    let compiler_options = resolution.compiler_options();
    assert_eq!(compiler_options.experimental_decorators, Some(true));
    assert_eq!(compiler_options.jsx, Some("react-jsx".to_string()));
    assert_eq!(compiler_options.jsx_factory, Some("h".to_string()));
    assert_eq!(compiler_options.jsx_fragment_factory, Some("Fragment".to_string()));
    assert_eq!(compiler_options.jsx_import_source, Some("xxx".to_string()));
}

// Template variable ${configDir} for substitution of config files directory path
// https://github.com/microsoft/TypeScript/pull/58042
#[test]
fn test_template_variable() {
    let f = super::fixture_root().join("tsconfig");
    let f2 = f.join("cases").join("paths_template_variable");

    #[rustfmt::skip]
    let pass = [
        (f2.clone(), "tsconfig.json", "foo", f2.join("src/foo.js")),
        (f2.clone(), "tsconfig_base_url1.json", "@/foo", f2.join("src/foo.js")),
        (f2.clone(), "tsconfig_base_url2.json", "@/foo", f2.join("src/foo.js")),
        (f2.clone(), "tsconfig_extends1.json", "foo", f2.join("src/foo.js")),
        (f2.clone(), "tsconfig_extends2.json", "foo", f2.join("src/foo.js")),
        (f2.clone(), "tsconfig_extends3.json", "foo", f2.join("src/foo.js")),
        (f2.clone(), "tsconfig_extends4.json", "foo", f2.join("src/foo.js")),
        (f.clone(), "tsconfig_template_variable1.json", "foo", f.join("src/foo.js")),
        (f.clone(), "tsconfig_template_variable2.json", "foo", f.join("src/foo.js")),
        (f.clone(), "tsconfig_template_variable3.json", "foo", f.join("src/foo.js")),
        (f.clone(), "tsconfig_template_variable4.json", "foo", f.join("src/foo.js")),
    ];

    for (dir, tsconfig, request, expected) in pass {
        let resolver = Resolver::new(ResolveOptions {
            tsconfig: Some(TsconfigOptions {
                config_file: dir.join(tsconfig),
                references: TsconfigReferences::Auto,
            }),
            ..ResolveOptions::default()
        });
        let resolved_path = resolver.resolve(&dir, request).map(|f| f.full_path());
        assert_eq!(resolved_path, Ok(expected), "{request} {tsconfig} {dir:?}");
    }
}

#[test]
fn test_paths_nested_base() {
    let f = super::fixture_root().join("tsconfig");
    let f2 = f.join("cases").join("paths-nested-base");

    #[rustfmt::skip]
    let pass = [
        (f2.join("other"), "tsconfig.json", "foo", f2.join("root/foo.ts")),
        (f2.join("root"), "tsconfig.json", "other/bar", f2.join("other/bar.ts")),
    ];

    for (dir, tsconfig, request, expected) in pass {
        let resolver = Resolver::new(ResolveOptions {
            tsconfig: Some(TsconfigOptions {
                config_file: dir.parent().unwrap().join(tsconfig),
                references: TsconfigReferences::Auto,
            }),
            ..ResolveOptions::default().with_extension(String::from(".ts"))
        });
        let resolved_path = resolver.resolve(&dir, request).map(|f| f.full_path());
        assert_eq!(resolved_path, Ok(expected), "{request} {tsconfig} {dir:?}");
    }
}

#[test]
fn test_parent_base_url() {
    let f = super::fixture_root().join("tsconfig");
    let f2 = f.join("cases").join("parent-base-url");

    #[rustfmt::skip]
    let pass = [
        (f2.join("test"), "tsconfig.json", ".", Err(ResolveError::NotFound(".".into()))),
        (f2.join("test"), "tsconfig.json", "index", Ok(f2.join("src/index.ts"))),
    ];

    for (dir, tsconfig, request, expected) in pass {
        let resolver = Resolver::new(ResolveOptions {
            tsconfig: Some(TsconfigOptions {
                config_file: dir.parent().unwrap().join(tsconfig),
                references: TsconfigReferences::Auto,
            }),
            ..ResolveOptions::default().with_extension(String::from(".ts"))
        });
        let resolved_path = resolver.resolve(&dir, request).map(|f| f.full_path());
        assert_eq!(resolved_path, expected, "{request} {tsconfig} {dir:?}");
    }
}

#[cfg(not(target_os = "windows"))] // MemoryFS's path separator is always `/` so the test will not pass in windows.
mod windows_test {
    use std::path::{Path, PathBuf};

    use super::super::memory_fs::MemoryFS;
    use crate::{
        ResolveError, ResolveOptions, ResolverGeneric, TsconfigOptions, TsconfigReferences,
    };

    struct OneTest {
        name: &'static str,
        tsconfig: String,
        package_json: Option<(PathBuf, String)>,
        main_fields: Option<Vec<String>>,
        existing_files: Vec<&'static str>,
        requested_module: &'static str,
        expected_path: &'static str,
        extensions: Vec<String>,
    }

    impl Default for OneTest {
        fn default() -> Self {
            Self {
                name: "",
                tsconfig: serde_json::json!({
                    "compilerOptions": {
                        "paths": {
                            "lib/*": ["location/*"]
                        }
                    }
                })
                .to_string(),
                package_json: None,
                main_fields: None,
                existing_files: vec![],
                requested_module: "",
                expected_path: "",
                extensions: vec![
                    ".js".into(),
                    ".json".into(),
                    ".node".into(),
                    ".ts".into(),
                    ".tsx".into(),
                ],
            }
        }
    }

    impl OneTest {
        fn resolver(&self, root: &Path) -> ResolverGeneric<MemoryFS> {
            let mut file_system = MemoryFS::default();

            file_system.add_file(&root.join("tsconfig.json"), &self.tsconfig);
            if let Some((path, package_json)) = &self.package_json {
                file_system.add_file(&root.join(path).join("package.json"), package_json);
            }
            for path in &self.existing_files {
                file_system.add_file(Path::new(path), "");
            }

            let mut options = ResolveOptions {
                extensions: self.extensions.clone(),
                tsconfig: Some(TsconfigOptions {
                    config_file: root.join("tsconfig.json"),
                    references: TsconfigReferences::Auto,
                }),
                ..ResolveOptions::default()
            };
            if let Some(main_fields) = &self.main_fields {
                options.main_fields.clone_from(main_fields);
            }

            ResolverGeneric::new_with_file_system(file_system, options)
        }
    }

    // Path matching tests from tsconfig-paths
    // * <https://github.com/dividab/tsconfig-paths/blob/master/src/__tests__/match-path-sync.test.ts>
    // * <https://github.com/dividab/tsconfig-paths/blob/master/src/__tests__/data/match-path-data.ts>
    #[test]
    fn match_path() {
        let pass = [
            OneTest {
                name: "should locate path that matches with star and exists",
                existing_files: vec!["/root/location/mylib/index.ts"],
                requested_module: "lib/mylib",
                expected_path: "/root/location/mylib/index.ts",
                ..OneTest::default()
            },
            OneTest {
                name: "should resolve to correct path when many are specified",
                tsconfig: serde_json::json!({
                    "compilerOptions": {
                        "paths": {
                            "lib/*": ["foo1/*", "foo2/*", "location/*", "foo3/*"],
                        }
                    }
                })
                .to_string(),
                existing_files: vec!["/root/location/mylib/index.ts"],
                requested_module: "lib/mylib",
                expected_path: "/root/location/mylib/index.ts",
                ..OneTest::default()
            },
            OneTest {
                name: "should locate path that matches with star and prioritize pattern with longest prefix",
                tsconfig: serde_json::json!({
                    "compilerOptions": {
                        "paths": {
                            "*": ["location/*"],
                            "lib/*": ["location/*"],
                        }
                    }
                })
                .to_string(),
                existing_files: vec![
                    "/root/location/lib/mylib/index.ts",
                    "/root/location/mylib/index.ts",
                ],
                requested_module: "lib/mylib",
                expected_path: "/root/location/mylib/index.ts",
                ..OneTest::default()
            },
            OneTest {
                name: "should locate path that matches with star and exists with extension",
                existing_files: vec!["/root/location/mylib.myext"],
                requested_module: "lib/mylib",
                extensions: vec![".js".into(), ".myext".into()],
                expected_path: "/root/location/mylib.myext",
                ..OneTest::default()
            },
            OneTest {
                name: "should resolve request with extension specified",
                existing_files: vec!["/root/location/test.jpg"],
                requested_module: "lib/test.jpg",
                expected_path: "/root/location/test.jpg",
                ..OneTest::default()
            },
            OneTest {
                name: "should locate path that matches without star and exists",
                tsconfig: serde_json::json!({
                    "compilerOptions": {
                        "paths": {
                            "lib/foo": ["location/foo"]
                        }
                    }
                })
                .to_string(),
                existing_files: vec!["/root/location/foo.ts"],
                requested_module: "lib/foo",
                expected_path: "/root/location/foo.ts",
                ..OneTest::default()
            },
            OneTest {
                name: "should resolve to parent folder when filename is in subfolder",
                existing_files: vec!["/root/location/mylib/index.ts"],
                requested_module: "lib/mylib",
                expected_path: "/root/location/mylib/index.ts",
                ..OneTest::default()
            },
            OneTest {
                name: "should resolve from main field in package.json",
                package_json: Some((
                    PathBuf::from("/root/location/mylib"),
                    serde_json::json!({
                        "main": "./kalle.ts"
                    })
                    .to_string(),
                )),
                existing_files: vec!["/root/location/mylib/kalle.ts"],
                requested_module: "lib/mylib",
                expected_path: "/root/location/mylib/kalle.ts",
                ..OneTest::default()
            },
            OneTest {
                name: "should resolve from main field in package.json (js)",
                package_json: Some((
                    PathBuf::from("/root/location/mylib.js"),
                    serde_json::json!({
                        "main": "./kalle.js"
                    })
                    .to_string(),
                )),
                existing_files: vec!["/root/location/mylib.js/kalle.js"],
                extensions: vec![".ts".into(), ".js".into()],
                requested_module: "lib/mylib.js",
                expected_path: "/root/location/mylib.js/kalle.js",
                ..OneTest::default()
            },
            OneTest {
                name: "should resolve from list of fields by priority in package.json",
                main_fields: Some(vec!["missing".into(), "browser".into(), "main".into()]),
                package_json: Some((
                    PathBuf::from("/root/location/mylibjs"),
                    serde_json::json!({
                        "main": "./main.js",
                        "browser": "./browser.js"
                    })
                    .to_string(),
                )),
                existing_files: vec![
                    "/root/location/mylibjs/main.js",
                    "/root/location/mylibjs/browser.js",
                ],
                extensions: vec![".ts".into(), ".js".into()],
                requested_module: "lib/mylibjs",
                expected_path: "/root/location/mylibjs/browser.js",
                ..OneTest::default()
            },
            OneTest {
                name: "should ignore field mappings to missing files in package.json",
                main_fields: Some(vec!["browser".into(), "main".into()]),
                package_json: Some((
                    PathBuf::from("/root/location/mylibjs"),
                    serde_json::json!({
                        "main": "./kalle.js",
                        "browser": "./nope.js"
                    })
                    .to_string(),
                )),
                existing_files: vec!["/root/location/mylibjs/kalle.js"],
                extensions: vec![".ts".into(), ".js".into()],
                requested_module: "lib/mylibjs",
                expected_path: "/root/location/mylibjs/kalle.js",
                ..OneTest::default()
            },
            // Tests that are not applicable:
            // name: "should resolve nested main fields"
            // name: "should ignore advanced field mappings in package.json"
            // name: "should resolve to with the help of baseUrl when not explicitly set"
            // name: "should not resolve with the help of baseUrl when asked not to"
            // name: "should resolve main file with cjs file extension"
            OneTest {
                name: "should resolve .ts from .js alias",
                tsconfig: serde_json::json!({
                    "compilerOptions": {
                        "paths": {
                            "@/*": ["src/*"]
                        }
                    }
                })
                .to_string(),
                existing_files: vec!["/root/src/foo.ts"],
                requested_module: "@/foo", // original data was "@/foo.ts" but I don't get why it is the case?
                expected_path: "/root/src/foo.ts", // original data was "/root/src/foo"
                ..OneTest::default()
            },
        ];

        let root = PathBuf::from("/root");

        for test in pass {
            let resolved_path =
                test.resolver(&root).resolve(&root, test.requested_module).map(|f| f.full_path());
            assert_eq!(resolved_path, Ok(PathBuf::from(test.expected_path)), "{}", test.name);
        }

        let fail = [
            OneTest {
                name: "should not locate path that does not match",
                tsconfig: serde_json::json!({
                    "compilerOptions": {
                        "paths": {
                            "lib/*": ["location/*"]
                        }
                    }
                })
                .to_string(),
                existing_files: vec!["/root/location/mylib"],
                requested_module: "lib/mylibjs",
                ..OneTest::default()
            },
            OneTest {
                name: "should not resolve typings file (index.d.ts)",
                existing_files: vec!["/root/location/mylib/index.d.ts"],
                requested_module: "lib/mylib",
                ..OneTest::default()
            },
        ];

        for test in fail {
            let resolved_path =
                test.resolver(&root).resolve(&root, test.requested_module).map(|f| f.full_path());
            assert_eq!(
                resolved_path,
                Err(ResolveError::NotFound(test.requested_module.into())),
                "{}",
                test.name
            );
        }
    }
}
