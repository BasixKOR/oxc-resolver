use std::{collections::HashMap, path::PathBuf, sync::Arc};

use fancy_regex::Regex;
use napi::Either;
use napi_derive::napi;

/// Module Resolution Options
///
/// Options are directly ported from [enhanced-resolve](https://github.com/webpack/enhanced-resolve#resolver-options).
///
/// See [webpack resolve](https://webpack.js.org/configuration/resolve/) for information and examples
#[derive(Debug, Clone)]
#[napi(object)]
pub struct NapiResolveOptions {
    /// Path to TypeScript configuration file.
    ///
    /// Default `None`
    pub tsconfig: Option<TsconfigOptions>,

    /// Alias for [ResolveOptions::alias] and [ResolveOptions::fallback].
    ///
    /// For the second value of the tuple, `None -> AliasValue::Ignore`, Some(String) ->
    /// AliasValue::Path(String)`
    /// Create aliases to import or require certain modules more easily.
    /// A trailing $ can also be added to the given object's keys to signify an exact match.
    /// Default `{}`
    pub alias: Option<HashMap<String, Vec<Option<String>>>>,

    /// A list of alias fields in description files.
    /// Specify a field, such as `browser`, to be parsed according to [this specification](https://github.com/defunctzombie/package-browser-field-spec).
    /// Can be a path to json object such as `["path", "to", "exports"]`.
    ///
    /// Default `[]`
    #[napi(ts_type = "(string | string[])[]")]
    pub alias_fields: Option<Vec<StrOrStrListType>>,

    /// Condition names for exports field which defines entry points of a package.
    /// The key order in the exports field is significant. During condition matching, earlier entries have higher priority and take precedence over later entries.
    ///
    /// Default `[]`
    pub condition_names: Option<Vec<String>>,

    /// If true, it will not allow extension-less files.
    /// So by default `require('./foo')` works if `./foo` has a `.js` extension,
    /// but with this enabled only `require('./foo.js')` will work.
    ///
    /// Default to `true` when [ResolveOptions::extensions] contains an empty string.
    /// Use `Some(false)` to disable the behavior.
    /// See <https://github.com/webpack/enhanced-resolve/pull/285>
    ///
    /// Default None, which is the same as `Some(false)` when the above empty rule is not applied.
    pub enforce_extension: Option<EnforceExtension>,

    /// A list of exports fields in description files.
    /// Can be a path to json object such as `["path", "to", "exports"]`.
    ///
    /// Default `[["exports"]]`.
    #[napi(ts_type = "(string | string[])[]")]
    pub exports_fields: Option<Vec<StrOrStrListType>>,

    /// Fields from `package.json` which are used to provide the internal requests of a package
    /// (requests starting with # are considered internal).
    ///
    /// Can be a path to a JSON object such as `["path", "to", "imports"]`.
    ///
    /// Default `[["imports"]]`.
    #[napi(ts_type = "(string | string[])[]")]
    pub imports_fields: Option<Vec<StrOrStrListType>>,

    /// An object which maps extension to extension aliases.
    ///
    /// Default `{}`
    pub extension_alias: Option<HashMap<String, Vec<String>>>,

    /// Attempt to resolve these extensions in order.
    /// If multiple files share the same name but have different extensions,
    /// will resolve the one with the extension listed first in the array and skip the rest.
    ///
    /// Default `[".js", ".json", ".node"]`
    pub extensions: Option<Vec<String>>,

    /// Redirect module requests when normal resolving fails.
    ///
    /// Default `{}`
    pub fallback: Option<HashMap<String, Vec<Option<String>>>>,

    /// Request passed to resolve is already fully specified and extensions or main files are not resolved for it (they are still resolved for internal requests).
    ///
    /// See also webpack configuration [resolve.fullySpecified](https://webpack.js.org/configuration/module/#resolvefullyspecified)
    ///
    /// Default `false`
    pub fully_specified: Option<bool>,

    /// A list of main fields in description files
    ///
    /// Default `["main"]`.
    #[napi(ts_type = "string | string[]")]
    pub main_fields: Option<StrOrStrListType>,

    /// The filename to be used while resolving directories.
    ///
    /// Default `["index"]`
    pub main_files: Option<Vec<String>>,

    /// A list of directories to resolve modules from, can be absolute path or folder name.
    ///
    /// Default `["node_modules"]`
    #[napi(ts_type = "string | string[]")]
    pub modules: Option<StrOrStrListType>,

    /// Resolve to a context instead of a file.
    ///
    /// Default `false`
    pub resolve_to_context: Option<bool>,

    /// Prefer to resolve module requests as relative requests instead of using modules from node_modules directories.
    ///
    /// Default `false`
    pub prefer_relative: Option<bool>,

    /// Prefer to resolve server-relative urls as absolute paths before falling back to resolve in ResolveOptions::roots.
    ///
    /// Default `false`
    pub prefer_absolute: Option<bool>,

    /// A list of resolve restrictions to restrict the paths that a request can be resolved on.
    ///
    /// Default `[]`
    pub restrictions: Option<Vec<Restriction>>,

    /// A list of directories where requests of server-relative URLs (starting with '/') are resolved.
    /// On non-Windows systems these requests are resolved as an absolute path first.
    ///
    /// Default `[]`
    pub roots: Option<Vec<String>>,

    /// Whether to resolve symlinks to their symlinked location.
    /// When enabled, symlinked resources are resolved to their real path, not their symlinked location.
    /// Note that this may cause module resolution to fail when using tools that symlink packages (like npm link).
    ///
    /// Default `true`
    pub symlinks: Option<bool>,

    /// Whether to parse [module.builtinModules](https://nodejs.org/api/module.html#modulebuiltinmodules) or not.
    /// For example, "zlib" will throw [crate::ResolveError::Builtin] when set to true.
    ///
    /// Default `false`
    pub builtin_modules: Option<bool>,

    /// Resolve [ResolveResult::moduleType].
    ///
    /// Default `false`
    pub module_type: Option<bool>,

    /// Allow `exports` field in `require('../directory')`.
    ///
    /// This is not part of the spec but some vite projects rely on this behavior.
    /// See
    /// * <https://github.com/vitejs/vite/pull/20252>
    /// * <https://github.com/nodejs/node/issues/58827>
    ///
    /// Default: `false`
    pub allow_package_exports_in_directory_resolve: Option<bool>,
}

#[napi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforceExtension {
    Auto,
    Enabled,
    Disabled,
}

impl EnforceExtension {
    pub fn is_auto(&self) -> bool {
        *self == Self::Auto
    }

    pub fn is_enabled(&self) -> bool {
        *self == Self::Enabled
    }

    pub fn is_disabled(&self) -> bool {
        *self == Self::Disabled
    }
}

/// Alias Value for [ResolveOptions::alias] and [ResolveOptions::fallback].
/// Use struct because napi don't support structured union now
#[napi(object)]
#[derive(Debug, Clone)]
pub struct Restriction {
    pub path: Option<String>,
    pub regex: Option<String>,
}

/// Tsconfig Options
///
/// Derived from [tsconfig-paths-webpack-plugin](https://github.com/dividab/tsconfig-paths-webpack-plugin#options)
#[napi(object)]
#[derive(Debug, Clone)]
pub struct TsconfigOptions {
    /// Allows you to specify where to find the TypeScript configuration file.
    /// You may provide
    /// * a relative path to the configuration file. It will be resolved relative to cwd.
    /// * an absolute path to the configuration file.
    pub config_file: String,

    /// Support for Typescript Project References.
    ///
    /// * `'auto'`: use the `references` field from tsconfig of `config_file`.
    /// * `string[]`: manually provided relative or absolute path.
    #[napi(ts_type = "'auto' | string[]")]
    pub references: Option<Either<String, Vec<String>>>,
}

impl From<Restriction> for oxc_resolver::Restriction {
    fn from(val: Restriction) -> Self {
        match (val.path, val.regex) {
            (None, None) => {
                panic!("Should specify path or regex")
            }
            (None, Some(regex)) => {
                let re = Regex::new(&regex).unwrap();
                oxc_resolver::Restriction::Fn(Arc::new(move |path| {
                    re.is_match(path.to_str().unwrap_or_default()).unwrap_or(false)
                }))
            }
            (Some(path), None) => oxc_resolver::Restriction::Path(PathBuf::from(path)),
            (Some(_), Some(_)) => {
                panic!("Restriction can't be path and regex at the same time")
            }
        }
    }
}

impl From<EnforceExtension> for oxc_resolver::EnforceExtension {
    fn from(val: EnforceExtension) -> Self {
        match val {
            EnforceExtension::Auto => oxc_resolver::EnforceExtension::Auto,
            EnforceExtension::Enabled => oxc_resolver::EnforceExtension::Enabled,
            EnforceExtension::Disabled => oxc_resolver::EnforceExtension::Disabled,
        }
    }
}

impl From<TsconfigOptions> for oxc_resolver::TsconfigOptions {
    fn from(val: TsconfigOptions) -> Self {
        oxc_resolver::TsconfigOptions {
            config_file: PathBuf::from(val.config_file),
            references: match val.references {
                Some(Either::A(string)) if string.as_str() == "auto" => {
                    oxc_resolver::TsconfigReferences::Auto
                }
                Some(Either::A(opt)) => {
                    panic!("`{}` is not a valid option for  tsconfig references", opt)
                }
                Some(Either::B(paths)) => oxc_resolver::TsconfigReferences::Paths(
                    paths.into_iter().map(PathBuf::from).collect::<Vec<_>>(),
                ),
                None => oxc_resolver::TsconfigReferences::Disabled,
            },
        }
    }
}

type StrOrStrListType = Either<String, Vec<String>>;
pub struct StrOrStrList(pub StrOrStrListType);

impl From<StrOrStrList> for Vec<String> {
    fn from(val: StrOrStrList) -> Self {
        match val {
            StrOrStrList(Either::A(s)) => Vec::from([s]),
            StrOrStrList(Either::B(a)) => a,
        }
    }
}
