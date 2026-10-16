use nms_copilot::completer::{CopilotCompleter, ModelCompletions};
use nms_copilot::commands::{Action, parse_line};
use reedline::Completer;
fn main() {
    for line in ["galaxy ", "galaxy Hilbert ", "galaxy \"Hilbert ", "galaxy \"Hilbert D", "galaxy Ei"] {
        let mut completer = CopilotCompleter::new(ModelCompletions {base_names: vec![], system_names: vec![]});
        let suggestions = completer.complete(line, line.len());
        let wanted = if line.ends_with("Ei") {"Eissentam"} else {"\"Hilbert Dimension\""};
        let suggestion = suggestions.iter().find(|s| s.value == wanted).expect("known reference name suggestion");
        let applied = format!("{}{}{}", &line[..suggestion.span.start], suggestion.value, &line[suggestion.span.end..]);
        let accepted = matches!(parse_line(&applied), Ok(Some(Action::Galaxy {selection: Some(ref name)})) if name == "Hilbert Dimension" || name == "Eissentam");
        println!("line={line:?}; span={}..{}; wanted_start=7; applied={applied:?}; parses_to_reference_name={accepted}", suggestion.span.start,suggestion.span.end);
    }
}
