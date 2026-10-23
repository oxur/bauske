//! Command parsing and output for the Relojban phonology executable.

mod args;
mod render;

use std::{
    ffi::OsString,
    io::{self, Write},
    process::ExitCode,
};

use args::{Cli, Command};
use clap::{Parser, error::ErrorKind};
use render::{
    RenderError, render_analysis, render_cluster, render_comparison_with_profile, render_inventory,
    render_profiles, render_prosody,
};

/// Runs the CLI from caller-supplied arguments and output streams.
pub fn run<I, T, W, E>(args: I, mut stdout: W, mut stderr: E) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
    W: Write,
    E: Write,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            let code = if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                0
            } else {
                2
            };
            let mut text = error.render().to_string();
            if !text.ends_with('\n') {
                text.push('\n');
            }
            let writer: &mut dyn Write = if error.use_stderr() {
                &mut stderr
            } else {
                &mut stdout
            };
            return match writer.write_all(text.as_bytes()) {
                Ok(()) => ExitCode::from(code),
                Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
                Err(_) => ExitCode::from(1),
            };
        }
    };
    match cli.command {
        Command::Inventory { kind } => output_result(render_inventory(kind, &mut stdout), 1),
        Command::Compare {
            left,
            right,
            profile,
        } => match render_comparison_with_profile(&left, &right, profile.into(), &mut stdout) {
            Ok(()) => ExitCode::SUCCESS,
            Err(RenderError::Io(error)) => output_result(Err(error), 1),
            Err(RenderError::Parse(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Compare(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Analysis(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Cluster(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Stress(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Realization(error)) => report_domain(&mut stderr, error),
        },
        Command::Profiles => output_result(render_profiles(&mut stdout), 1),
        Command::Analyze {
            word,
            kind,
            syllabic_at,
        } => match render_analysis(&word, kind, &syllabic_at, &mut stdout) {
            Ok(()) => ExitCode::SUCCESS,
            Err(RenderError::Io(error)) => output_result(Err(error), 1),
            Err(RenderError::Analysis(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Cluster(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Parse(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Compare(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Stress(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Realization(error)) => report_domain(&mut stderr, error),
        },
        Command::Cluster {
            cluster,
            position,
            kind,
        } => match render_cluster(&cluster, position, kind, &mut stdout) {
            Ok(()) => ExitCode::SUCCESS,
            Err(RenderError::Io(error)) => output_result(Err(error), 1),
            Err(RenderError::Cluster(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Analysis(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Parse(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Compare(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Stress(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Realization(error)) => report_domain(&mut stderr, error),
        },
        Command::Prosody {
            word,
            kind,
            syllabic_at,
            buffer_after,
            syllables,
            primary,
            unstressed,
            following,
        } => match render_prosody(
            render::ProsodyRequest {
                input: &word,
                kind,
                syllabic_positions: &syllabic_at,
                buffer_after: &buffer_after,
                mode: syllables,
                primary,
                unstressed,
                following,
            },
            &mut stdout,
        ) {
            Ok(()) => ExitCode::SUCCESS,
            Err(RenderError::Io(error)) => output_result(Err(error), 1),
            Err(RenderError::Analysis(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Stress(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Realization(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Parse(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Compare(error)) => report_domain(&mut stderr, error),
            Err(RenderError::Cluster(error)) => report_domain(&mut stderr, error),
        },
    }
}

fn output_result(result: io::Result<()>, failure_code: u8) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(failure_code),
    }
}

fn report_domain<W: Write>(stderr: &mut W, error: impl std::fmt::Display) -> ExitCode {
    match writeln!(stderr, "error: {error}") {
        Ok(()) => ExitCode::from(2),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    struct BrokenWriter(io::ErrorKind);
    impl Write for BrokenWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(self.0, "injected writer failure"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn non_broken_output_failure_returns_one() {
        let mut stderr = Vec::new();
        let code = run(
            ["relojban", "inventory"],
            BrokenWriter(io::ErrorKind::Other),
            &mut stderr,
        );
        assert_eq!(code, ExitCode::from(1));
    }

    #[test]
    fn prosody_profiles_and_alternate_compare_writer_failures_return_one() {
        for args in [
            vec!["relojban", "prosody", "klama", "--kind", "brivla"],
            vec!["relojban", "profiles"],
            vec![
                "relojban",
                "compare",
                "e",
                "o",
                "--profile",
                "close-mid-e-v1",
            ],
        ] {
            let code = run(
                args.iter().copied(),
                BrokenWriter(io::ErrorKind::Other),
                Vec::new(),
            );
            assert_eq!(code, ExitCode::from(1), "args={args:?}");
        }
    }

    #[test]
    fn broken_pipe_is_a_successful_exit() {
        let code = run(
            ["relojban", "inventory"],
            BrokenWriter(io::ErrorKind::BrokenPipe),
            Vec::new(),
        );
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn help_writer_failure_is_handled() {
        let code = run(
            ["relojban", "--help"],
            BrokenWriter(io::ErrorKind::BrokenPipe),
            Vec::new(),
        );
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn renderers_surface_io_failures_at_the_writer_boundary() {
        let ordinary = crate::render::render_inventory(
            crate::args::InventoryKind::Alphabet,
            BrokenWriter(io::ErrorKind::Other),
        )
        .unwrap_err();
        assert_eq!(ordinary.kind(), io::ErrorKind::Other);

        let broken = crate::render::render_comparison_with_profile(
            "p",
            "b",
            bauske_phonology::PronunciationProfile::RepresentativesV1,
            BrokenWriter(io::ErrorKind::BrokenPipe),
        )
        .unwrap_err();
        assert!(
            matches!(broken, RenderError::Io(error) if error.kind() == io::ErrorKind::BrokenPipe)
        );
    }
    #[test]
    fn new_renderers_surface_io_failures_and_commands_handle_broken_pipe() {
        let analysis = crate::render::render_analysis(
            "ai",
            crate::args::WordKindArg::Cmavo,
            &[],
            BrokenWriter(io::ErrorKind::Other),
        )
        .unwrap_err();
        assert!(matches!(analysis, RenderError::Io(error) if error.kind() == io::ErrorKind::Other));

        let cluster = crate::render::render_cluster(
            "bf",
            crate::args::ClusterPositionArg::Medial,
            crate::args::WordKindArg::Brivla,
            BrokenWriter(io::ErrorKind::BrokenPipe),
        )
        .unwrap_err();
        assert!(
            matches!(cluster, RenderError::Io(error) if error.kind() == io::ErrorKind::BrokenPipe)
        );

        let prosody = crate::render::render_prosody(
            crate::render::ProsodyRequest {
                input: "klama",
                kind: crate::args::WordKindArg::Brivla,
                syllabic_positions: &[],
                buffer_after: &[],
                mode: crate::args::SyllablesArg::Canonical,
                primary: None,
                unstressed: false,
                following: crate::args::FollowingArg::Unspecified,
            },
            BrokenWriter(io::ErrorKind::Other),
        )
        .unwrap_err();
        assert!(matches!(prosody, RenderError::Io(error) if error.kind() == io::ErrorKind::Other));

        let profiles =
            crate::render::render_profiles(BrokenWriter(io::ErrorKind::Other)).unwrap_err();
        assert_eq!(profiles.kind(), io::ErrorKind::Other);

        let comparison = crate::render::render_comparison_with_profile(
            "p",
            "b",
            bauske_phonology::PronunciationProfile::RepresentativesV1,
            BrokenWriter(io::ErrorKind::Other),
        )
        .unwrap_err();
        assert!(
            matches!(comparison, RenderError::Io(error) if error.kind() == io::ErrorKind::Other)
        );

        let analyze_code = run(
            ["relojban", "analyze", "ai", "--kind", "cmavo"],
            BrokenWriter(io::ErrorKind::BrokenPipe),
            Vec::new(),
        );
        assert_eq!(analyze_code, ExitCode::SUCCESS);
        let cluster_code = run(
            [
                "relojban",
                "cluster",
                "bf",
                "--position",
                "medial",
                "--kind",
                "brivla",
            ],
            BrokenWriter(io::ErrorKind::BrokenPipe),
            Vec::new(),
        );
        assert_eq!(cluster_code, ExitCode::SUCCESS);

        let prosody_code = run(
            ["relojban", "prosody", "klama", "--kind", "brivla"],
            BrokenWriter(io::ErrorKind::BrokenPipe),
            Vec::new(),
        );
        assert_eq!(prosody_code, ExitCode::SUCCESS);
        let profiles_code = run(
            ["relojban", "profiles"],
            BrokenWriter(io::ErrorKind::BrokenPipe),
            Vec::new(),
        );
        assert_eq!(profiles_code, ExitCode::SUCCESS);
        let compare_code = run(
            ["relojban", "compare", "p", "b"],
            BrokenWriter(io::ErrorKind::BrokenPipe),
            Vec::new(),
        );
        assert_eq!(compare_code, ExitCode::SUCCESS);
    }

    #[test]
    fn prosody_cli_reports_hint_resolution_and_realization_policy() {
        let mut output = Vec::new();
        let code = run(
            [
                "relojban",
                "prosody",
                "gas,tro",
                "--kind",
                "brivla",
                "--syllables",
                "hints",
            ],
            &mut output,
            Vec::new(),
        );
        assert_eq!(code, ExitCode::SUCCESS);
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("syllabification_resolved\ttrue"));
        assert!(output.contains("realization_policy\tbuffer-plan-v1"));

        for (word, expected) in [
            ("a,'i", "syllabification_resolved\ttrue"),
            (
                "a',i",
                "syllable_limitation\tsyllables.hint-conflicts-with-apostrophe",
            ),
        ] {
            let mut output = Vec::new();
            let code = run(
                [
                    "relojban",
                    "prosody",
                    word,
                    "--kind",
                    "cmavo",
                    "--syllables",
                    "hints",
                ],
                &mut output,
                Vec::new(),
            );
            assert_eq!(code, ExitCode::SUCCESS, "{word}");
            assert!(
                String::from_utf8(output).unwrap().contains(expected),
                "{word}"
            );
        }
    }
}
