#[macro_use]
extern crate cargotest;
extern crate hamcrest;

use std::io::Read;
use std::fs::File;

use cargotest::support::{project, execs};
use cargotest::support::registry::Package;
use hamcrest::{assert_that, existing_file, existing_dir, is_not};

#[test]
fn simple_explicit() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [workspace]
            members = ["bar"]
        "#)
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []
            workspace = ".."
        "#)
        .file("bar/src/main.rs", "fn main() {}");
    p.build();

    assert_that(p.cargo("build"), execs().with_status(0));
    assert_that(&p.bin("foo"), existing_file());
    assert_that(&p.bin("bar"), is_not(existing_file()));

    assert_that(p.cargo("build").cwd(p.root().join("bar")),
                execs().with_status(0));
    assert_that(&p.bin("foo"), existing_file());
    assert_that(&p.bin("bar"), existing_file());

    assert_that(&p.root().join("Cargo.lock"), existing_file());
    assert_that(&p.root().join("bar/Cargo.lock"), is_not(existing_file()));
}

#[test]
fn inferred_root() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [workspace]
            members = ["bar"]
        "#)
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []
        "#)
        .file("bar/src/main.rs", "fn main() {}");
    p.build();

    assert_that(p.cargo("build"), execs().with_status(0));
    assert_that(&p.bin("foo"), existing_file());
    assert_that(&p.bin("bar"), is_not(existing_file()));

    assert_that(p.cargo("build").cwd(p.root().join("bar")),
                execs().with_status(0));
    assert_that(&p.bin("foo"), existing_file());
    assert_that(&p.bin("bar"), existing_file());

    assert_that(&p.root().join("Cargo.lock"), existing_file());
    assert_that(&p.root().join("bar/Cargo.lock"), is_not(existing_file()));
}

#[test]
fn inferred_path_dep() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [dependencies]
            bar = { path = "bar" }

            [workspace]
        "#)
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []
        "#)
        .file("bar/src/main.rs", "fn main() {}")
        .file("bar/src/lib.rs", "");
    p.build();

    assert_that(p.cargo("build"), execs().with_status(0));
    assert_that(&p.bin("foo"), existing_file());
    assert_that(&p.bin("bar"), is_not(existing_file()));

    assert_that(p.cargo("build").cwd(p.root().join("bar")),
                execs().with_status(0));
    assert_that(&p.bin("foo"), existing_file());
    assert_that(&p.bin("bar"), existing_file());

    assert_that(&p.root().join("Cargo.lock"), existing_file());
    assert_that(&p.root().join("bar/Cargo.lock"), is_not(existing_file()));
}

#[test]
fn transitive_path_dep() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [dependencies]
            bar = { path = "bar" }

            [workspace]
        "#)
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []

            [dependencies]
            baz = { path = "../baz" }
        "#)
        .file("bar/src/main.rs", "fn main() {}")
        .file("bar/src/lib.rs", "")
        .file("baz/Cargo.toml", r#"
            [project]
            name = "baz"
            version = "0.1.0"
            authors = []
        "#)
        .file("baz/src/main.rs", "fn main() {}")
        .file("baz/src/lib.rs", "");
    p.build();

    assert_that(p.cargo("build"), execs().with_status(0));
    assert_that(&p.bin("foo"), existing_file());
    assert_that(&p.bin("bar"), is_not(existing_file()));
    assert_that(&p.bin("baz"), is_not(existing_file()));

    assert_that(p.cargo("build").cwd(p.root().join("bar")),
                execs().with_status(0));
    assert_that(&p.bin("foo"), existing_file());
    assert_that(&p.bin("bar"), existing_file());
    assert_that(&p.bin("baz"), is_not(existing_file()));

    assert_that(p.cargo("build").cwd(p.root().join("baz")),
                execs().with_status(0));
    assert_that(&p.bin("foo"), existing_file());
    assert_that(&p.bin("bar"), existing_file());
    assert_that(&p.bin("baz"), existing_file());

    assert_that(&p.root().join("Cargo.lock"), existing_file());
    assert_that(&p.root().join("bar/Cargo.lock"), is_not(existing_file()));
    assert_that(&p.root().join("baz/Cargo.lock"), is_not(existing_file()));
}

#[test]
fn parent_pointer_works() {
    let p = project("foo")
        .file("foo/Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [dependencies]
            bar = { path = "../bar" }

            [workspace]
        "#)
        .file("foo/src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []
            workspace = "../foo"
        "#)
        .file("bar/src/main.rs", "fn main() {}")
        .file("bar/src/lib.rs", "");
    p.build();

    assert_that(p.cargo("build").cwd(p.root().join("foo")),
                execs().with_status(0));
    assert_that(p.cargo("build").cwd(p.root().join("bar")),
                execs().with_status(0));
    assert_that(&p.root().join("foo/Cargo.lock"), existing_file());
    assert_that(&p.root().join("bar/Cargo.lock"), is_not(existing_file()));
}

#[test]
fn same_names_in_workspace() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [workspace]
            members = ["bar"]
        "#)
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []
            workspace = ".."
        "#)
        .file("bar/src/main.rs", "fn main() {}");
    p.build();

    assert_that(p.cargo("build"),
                execs().with_status(101)
                       .with_stderr("\
error: two packages named `foo` in this workspace:
- [..]Cargo.toml
- [..]Cargo.toml
"));
}

#[test]
fn parent_doesnt_point_to_child() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [workspace]
        "#)
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []
        "#)
        .file("bar/src/main.rs", "fn main() {}");
    p.build();

    assert_that(p.cargo("build").cwd(p.root().join("bar")),
                execs().with_status(101)
                       .with_stderr("\
error: current package believes it's in a workspace when it's not:
current: [..]Cargo.toml
workspace: [..]Cargo.toml

this may be fixable [..]
"));
}

#[test]
fn invalid_parent_pointer() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []
            workspace = "foo"
        "#)
        .file("src/main.rs", "fn main() {}");
    p.build();

    assert_that(p.cargo("build"),
                execs().with_status(101)
                       .with_stderr("\
error: failed to read `[..]Cargo.toml`

Caused by:
  [..]
"));
}

#[test]
fn invalid_members() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [workspace]
            members = ["foo"]
        "#)
        .file("src/main.rs", "fn main() {}");
    p.build();

    assert_that(p.cargo("build"),
                execs().with_status(101)
                       .with_stderr("\
error: failed to read `[..]Cargo.toml`

Caused by:
  [..]
"));
}

#[test]
fn bare_workspace_ok() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [workspace]
        "#)
        .file("src/main.rs", "fn main() {}");
    p.build();

    assert_that(p.cargo("build"), execs().with_status(0));
}

#[test]
fn two_roots() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [workspace]
            members = ["bar"]
        "#)
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []

            [workspace]
            members = [".."]
        "#)
        .file("bar/src/main.rs", "fn main() {}");
    p.build();

    assert_that(p.cargo("build"),
                execs().with_status(101)
                       .with_stderr("\
error: multiple workspace roots found in the same workspace:
  [..]
  [..]
"));
}

#[test]
fn workspace_isnt_root() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []
            workspace = "bar"
        "#)
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []
        "#)
        .file("bar/src/main.rs", "fn main() {}");
    p.build();

    assert_that(p.cargo("build"),
                execs().with_status(101)
                       .with_stderr("\
error: root of a workspace inferred but wasn't a root: [..]
"));
}

#[test]
fn dangling_member() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [workspace]
            members = ["bar"]
        "#)
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []
            workspace = "../baz"
        "#)
        .file("bar/src/main.rs", "fn main() {}")
        .file("baz/Cargo.toml", r#"
            [project]
            name = "baz"
            version = "0.1.0"
            authors = []
            workspace = "../baz"
        "#)
        .file("baz/src/main.rs", "fn main() {}");
    p.build();

    assert_that(p.cargo("build"),
                execs().with_status(101)
                       .with_stderr("\
error: package `[..]` is a member of the wrong workspace
expected: [..]
actual: [..]
"));
}

#[test]
fn cycle() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []
            workspace = "bar"
        "#)
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []
            workspace = ".."
        "#)
        .file("bar/src/main.rs", "fn main() {}");
    p.build();

    assert_that(p.cargo("build"),
                execs().with_status(101));
}

#[test]
fn share_dependencies() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [dependencies]
            dep1 = "0.1"

            [workspace]
            members = ["bar"]
        "#)
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []

            [dependencies]
            dep1 = "< 0.1.5"
        "#)
        .file("bar/src/main.rs", "fn main() {}");
    p.build();

    Package::new("dep1", "0.1.3").publish();
    Package::new("dep1", "0.1.8").publish();

    assert_that(p.cargo("build"),
                execs().with_status(0)
                       .with_stderr("\
[UPDATING] registry `[..]`
[DOWNLOADING] dep1 v0.1.3 ([..])
[COMPILING] dep1 v0.1.3 ([..])
[COMPILING] foo v0.1.0 ([..])
"));
}

#[test]
fn fetch_fetches_all() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [workspace]
            members = ["bar"]
        "#)
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []

            [dependencies]
            dep1 = "*"
        "#)
        .file("bar/src/main.rs", "fn main() {}");
    p.build();

    Package::new("dep1", "0.1.3").publish();

    assert_that(p.cargo("fetch"),
                execs().with_status(0)
                       .with_stderr("\
[UPDATING] registry `[..]`
[DOWNLOADING] dep1 v0.1.3 ([..])
"));
}

#[test]
fn lock_works_for_everyone() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [dependencies]
            dep2 = "0.1"

            [workspace]
            members = ["bar"]
        "#)
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []

            [dependencies]
            dep1 = "0.1"
        "#)
        .file("bar/src/main.rs", "fn main() {}");
    p.build();

    Package::new("dep1", "0.1.0").publish();
    Package::new("dep2", "0.1.0").publish();

    assert_that(p.cargo("generate-lockfile"),
                execs().with_status(0)
                       .with_stderr("\
[UPDATING] registry `[..]`
"));

    Package::new("dep1", "0.1.1").publish();
    Package::new("dep2", "0.1.1").publish();

    assert_that(p.cargo("build"),
                execs().with_status(0)
                       .with_stderr("\
[DOWNLOADING] dep2 v0.1.0 ([..])
[COMPILING] dep2 v0.1.0 ([..])
[COMPILING] foo v0.1.0 ([..])
"));

    assert_that(p.cargo("build").cwd(p.root().join("bar")),
                execs().with_status(0)
                       .with_stderr("\
[DOWNLOADING] dep1 v0.1.0 ([..])
[COMPILING] dep1 v0.1.0 ([..])
[COMPILING] bar v0.1.0 ([..])
"));
}

#[test]
fn virtual_works() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [workspace]
            members = ["bar"]
        "#)
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []
        "#)
        .file("bar/src/main.rs", "fn main() {}");
    p.build();
    assert_that(p.cargo("build").cwd(p.root().join("bar")),
                execs().with_status(0));
    assert_that(&p.root().join("Cargo.lock"), existing_file());
    assert_that(&p.bin("bar"), existing_file());
    assert_that(&p.root().join("bar/Cargo.lock"), is_not(existing_file()));
}

#[test]
fn virtual_misconfigure() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [workspace]
        "#)
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []
        "#)
        .file("bar/src/main.rs", "fn main() {}");
    p.build();
    assert_that(p.cargo("build").cwd(p.root().join("bar")),
                execs().with_status(101)
                       .with_stderr("\
error: current package believes it's in a workspace when it's not:
current: [..]bar[..]Cargo.toml
workspace: [..]Cargo.toml

this may be fixable by adding `bar` to the `workspace.members` array of the \
manifest located at: [..]
"));
}

#[test]
fn virtual_build() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [workspace]
        "#)
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []
        "#)
        .file("bar/src/main.rs", "fn main() {}");
    p.build();
    assert_that(p.cargo("build"),
                execs().with_status(101)
                       .with_stderr("\
error: manifest path `[..]` is a virtual manifest, but this command \
requires running against an actual package in this workspace
"));
}

#[test]
fn include_virtual() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.1.0"
            authors = []
            [workspace]
            members = ["bar"]
        "#)
        .file("src/main.rs", "")
        .file("bar/Cargo.toml", r#"
            [workspace]
        "#);
    p.build();
    assert_that(p.cargo("build"),
                execs().with_status(101)
                       .with_stderr("\
error: multiple workspace roots found in the same workspace:
  [..]
  [..]
"));
}

#[test]
fn members_include_path_deps() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [workspace]
            members = ["p1"]

            [dependencies]
            p3 = { path = "p3" }
        "#)
        .file("src/lib.rs", "")
        .file("p1/Cargo.toml", r#"
            [project]
            name = "p1"
            version = "0.1.0"
            authors = []

            [dependencies]
            p2 = { path = "../p2" }
        "#)
        .file("p1/src/lib.rs", "")
        .file("p2/Cargo.toml", r#"
            [project]
            name = "p2"
            version = "0.1.0"
            authors = []
        "#)
        .file("p2/src/lib.rs", "")
        .file("p3/Cargo.toml", r#"
            [project]
            name = "p3"
            version = "0.1.0"
            authors = []
        "#)
        .file("p3/src/lib.rs", "");
    p.build();

    assert_that(p.cargo("build").cwd(p.root().join("p1")),
                execs().with_status(0));
    assert_that(p.cargo("build").cwd(p.root().join("p2")),
                execs().with_status(0));
    assert_that(p.cargo("build").cwd(p.root().join("p3")),
                execs().with_status(0));
    assert_that(p.cargo("build"),
                execs().with_status(0));

    assert_that(&p.root().join("target"), existing_dir());
    assert_that(&p.root().join("p1/target"), is_not(existing_dir()));
    assert_that(&p.root().join("p2/target"), is_not(existing_dir()));
    assert_that(&p.root().join("p3/target"), is_not(existing_dir()));
}

#[test]
fn new_warns_you_this_will_not_work() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [workspace]
        "#)
        .file("src/lib.rs", "");
    p.build();

    assert_that(p.cargo("new").arg("bar").env("USER", "foo"),
                execs().with_status(0)
                       .with_stderr("\
warning: compiling this new crate may not work due to invalid workspace \
configuration

current package believes it's in a workspace when it's not:
current: [..]
workspace: [..]

this may be fixable by ensuring that this crate is depended on by the workspace \
root: [..]
"));
}

#[test]
fn lock_doesnt_change_depending_on_crate() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.1.0"
            authors = []

            [workspace]
            members = ['baz']

            [dependencies]
            foo = "*"
        "#)
        .file("src/lib.rs", "")
        .file("baz/Cargo.toml", r#"
            [project]
            name = "baz"
            version = "0.1.0"
            authors = []

            [dependencies]
            bar = "*"
        "#)
        .file("baz/src/lib.rs", "");
    p.build();

    Package::new("foo", "1.0.0").publish();
    Package::new("bar", "1.0.0").publish();

    assert_that(p.cargo("build"),
                execs().with_status(0));

    let mut lockfile = String::new();
    t!(t!(File::open(p.root().join("Cargo.lock"))).read_to_string(&mut lockfile));

    assert_that(p.cargo("build").cwd(p.root().join("baz")),
                execs().with_status(0));

    let mut lockfile2 = String::new();
    t!(t!(File::open(p.root().join("Cargo.lock"))).read_to_string(&mut lockfile2));

    assert_eq!(lockfile, lockfile2);
}
