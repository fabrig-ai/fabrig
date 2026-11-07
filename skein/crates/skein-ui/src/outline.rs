//! The page's landmark outline (skein's ui.md, section 14).
use crate::{Role, Shortcut, State, Tree};
use skein_lib::List;

/// The first node in preorder breaking the outline, or zero for a missing node.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Fault {
    pub rule: Rule,
    pub node: u32,
}

/// One rule of the page's outline.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rule {
    /// A root that is not a page.
    PageRoot,
    /// A page child outside a landmark or dialog.
    InsideLandmark,
    /// A second main, or no main.
    OneMain,
    /// A second banner.
    OneBanner,
    /// A second content info.
    OneContentInfo,
    /// A shortcut already named by an earlier node.
    ShortcutOnce,
    /// Escape named by a node outside every dialog and every form.
    EscapeInDialogOrForm,
    /// An options menu has no preceding text field in its innermost form.
    OptionsAfterField,
}

/// Check roles, region names and shortcuts, without reading words.
#[must_use]
pub fn check(tree: &Tree) -> Option<Fault> {
    let fault = landmarks(tree);
    let length = u32::try_from(tree.nodes().len()).expect("tree length fits u32");
    let mut seen: List<Shortcut> = List::with_capacity(length);
    let mut dialog_end = 0;
    let mut form_end = 0;
    for index in 0..length {
        let node = tree.get(index).expect("index within tree");
        if node.role == Role::Dialog {
            dialog_end = dialog_end.max(index.checked_add(node.size).expect("dialog end fits u32"));
        }
        if node.role == Role::Form {
            form_end = form_end.max(index.checked_add(node.size).expect("form end fits u32"));
        }
        if node.states.as_slice().contains(&State::Options) && !options_after_field(tree, index) {
            if let Some(earlier) = fault
                && earlier.node <= index
            {
                return Some(earlier);
            }
            return Some(Fault { rule: Rule::OptionsAfterField, node: index });
        }
        if let Some(shortcut) = node.shortcut {
            if shortcut == Shortcut::ESCAPE && index >= dialog_end && index >= form_end {
                if let Some(earlier) = fault
                    && earlier.node <= index
                {
                    return Some(earlier);
                }
                return Some(Fault { rule: Rule::EscapeInDialogOrForm, node: index });
            }
            for previous in &seen {
                if *previous == shortcut {
                    if let Some(earlier) = fault
                        && earlier.node <= index
                    {
                        return Some(earlier);
                    }
                    return Some(Fault { rule: Rule::ShortcutOnce, node: index });
                }
            }
            seen.push(shortcut).expect("at most one shortcut per node");
        }
    }
    fault
}

fn landmarks(tree: &Tree) -> Option<Fault> {
    let Some(root) = tree.get(0) else {
        return Some(Fault { rule: Rule::PageRoot, node: 0 });
    };
    if root.role != Role::Page {
        return Some(Fault { rule: Rule::PageRoot, node: 0 });
    }
    let mut main = false;
    let mut banner = false;
    let mut content_info = false;
    let mut next = 1;
    let mut fault = None;
    let length = u32::try_from(tree.nodes().len()).expect("tree length fits u32");
    for index in 1..length {
        let node = tree.get(index).expect("page descendant exists");
        let named = match &node.name {
            Some(name) => !name.is_empty(),
            None => false,
        };
        let mut rule = None;
        if index == next {
            next = index.checked_add(node.size).expect("child end fits");
            if !(node.role == Role::Banner
                || node.role == Role::Navigation
                || node.role == Role::Main
                || node.role == Role::Complementary
                || node.role == Role::ContentInfo
                || node.role == Role::Form
                || node.role == Role::Search
                || node.role == Role::Dialog
                || (node.role == Role::Region && named))
            {
                rule = Some(Rule::InsideLandmark);
            }
        }
        if node.role == Role::Main {
            if main {
                rule = Some(Rule::OneMain);
            }
            main = true;
        } else if node.role == Role::Banner {
            if banner {
                rule = Some(Rule::OneBanner);
            }
            banner = true;
        } else if node.role == Role::ContentInfo {
            if content_info {
                rule = Some(Rule::OneContentInfo);
            }
            content_info = true;
        }
        if fault.is_none()
            && let Some(rule) = rule
        {
            fault = Some(Fault { rule, node: index });
        }
    }
    if !main {
        return Some(Fault { rule: Rule::OneMain, node: 0 });
    }
    fault
}

fn options_after_field(tree: &Tree, menu: u32) -> bool {
    let mut form = None;
    for index in 0..menu {
        let node = tree.get(index).expect("index before menu");
        let end = index.checked_add(node.size).expect("subtree end fits u32");
        if node.role == Role::Form && menu < end {
            form = Some(index);
        }
    }
    let Some(form) = form else {
        return false;
    };
    let mut nested_end = 0;
    for index in form.checked_add(1).expect("form precedes menu")..menu {
        if index < nested_end {
            continue;
        }
        let node = tree.get(index).expect("index before menu");
        if node.role == Role::Form {
            nested_end = index.checked_add(node.size).expect("subtree end fits u32");
        } else if node.role.is_text_field() {
            return true;
        }
    }
    false
}
