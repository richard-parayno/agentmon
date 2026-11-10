//! The menu bar item (an AppKit `NSStatusItem`). Rune's `Tray` has no macOS backend yet, so the
//! app owns the item and hands clicks to the Rune popover.
//!
//! Everything here runs on the main thread: `install` from `on_ready`, `set_title` from
//! `AppHandle::update` (which runs on the UI thread), clicks from AppKit's target-action.

use std::cell::RefCell;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{NSScreen, NSStatusBar, NSStatusItem, NSVariableStatusItemLength};
use objc2_foundation::{NSRect, NSString};

/// Where the item sits, in AppKit screen points (origin at the bottom left of the primary
/// screen), plus what's needed to place a window under it.
#[derive(Clone, Copy, Debug)]
pub struct Anchor {
    pub frame: NSRect,
    /// The height of the primary screen, to flip y to top-left coordinates.
    pub primary_height: f64,
    pub scale: f64,
    /// The visible frame of the screen the item is on, to keep the popover inside it.
    pub screen: NSRect,
}

struct Installed {
    item: Retained<NSStatusItem>,
    _target: Retained<Target>,
    on_click: Box<dyn Fn(Anchor)>,
}

thread_local! {
    static ITEM: RefCell<Option<Installed>> = const { RefCell::new(None) };
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "AgentmonStatusTarget"]
    struct Target;

    impl Target {
        #[unsafe(method(clicked:))]
        fn clicked(&self, _sender: Option<&AnyObject>) {
            ITEM.with(|i| {
                if let Some(i) = i.borrow().as_ref()
                    && let Some(anchor) = anchor(&i.item, self.mtm())
                {
                    (i.on_click)(anchor);
                }
            });
        }
    }
);

impl Target {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![Self::alloc(mtm), init] }
    }
}

/// Adds the item to the menu bar. A second call replaces the first.
pub fn install(title: &str, on_click: impl Fn(Anchor) + 'static) {
    let Some(mtm) = MainThreadMarker::new() else {
        eprintln!("agentmon: the status item must be installed on the main thread");
        return;
    };
    let item = NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
    let target = Target::new(mtm);
    if let Some(button) = item.button(mtm) {
        button.setTitle(&NSString::from_str(title));
        unsafe {
            button.setTarget(Some(&target));
            button.setAction(Some(sel!(clicked:)));
        }
    }
    ITEM.with(|i| {
        *i.borrow_mut() = Some(Installed {
            item,
            _target: target,
            on_click: Box::new(on_click),
        })
    });
}

pub fn set_title(title: &str) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    ITEM.with(|i| {
        if let Some(button) = i.borrow().as_ref().and_then(|i| i.item.button(mtm)) {
            button.setTitle(&NSString::from_str(title));
        }
    });
}

fn anchor(item: &NSStatusItem, mtm: MainThreadMarker) -> Option<Anchor> {
    let window = item.button(mtm)?.window()?;
    let frame = window.frame();
    let primary_height = NSScreen::screens(mtm).firstObject()?.frame().size.height;
    let screen = window.screen().map(|s| s.visibleFrame()).unwrap_or(frame);
    Some(Anchor {
        frame,
        primary_height,
        scale: window.backingScaleFactor(),
        screen,
    })
}

impl Anchor {
    /// The top-left corner, in physical desktop pixels, for a window `size` points wide that
    /// hangs centred under the item, kept inside the item's screen.
    pub fn popover_origin(&self, size: (f32, f32)) -> (i32, i32) {
        let (w, gap, margin) = (size.0 as f64, 4.0, 8.0);
        let min_x = self.screen.origin.x + margin;
        let max_x = (self.screen.origin.x + self.screen.size.width - w - margin).max(min_x);
        let x = (self.frame.origin.x + self.frame.size.width / 2.0 - w / 2.0).clamp(min_x, max_x);
        // AppKit's y grows upward from the primary screen's bottom; the window's grows downward.
        let top = self.primary_height - self.frame.origin.y + gap;
        (
            (x * self.scale).round() as i32,
            (top * self.scale).round() as i32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2_foundation::{NSPoint, NSSize};

    fn rect(x: f64, y: f64, w: f64, h: f64) -> NSRect {
        NSRect::new(NSPoint::new(x, y), NSSize::new(w, h))
    }

    #[test]
    fn the_popover_hangs_under_the_item_and_stays_on_screen() {
        // A 1512×982 pt screen at 2×; the menu bar is 33 pt, its item 60 pt wide at x = 1000.
        let screen = rect(0.0, 0.0, 1512.0, 949.0);
        let a = Anchor {
            frame: rect(1000.0, 949.0, 60.0, 33.0),
            primary_height: 982.0,
            scale: 2.0,
            screen,
        };
        // Centred: 1030 - 180 = 850 pt; top 982 - 949 + 4 = 37 pt.
        assert_eq!(a.popover_origin((360.0, 480.0)), (1700, 74));

        // Near the right edge it's pushed back inside the 8 pt margin: 1512 - 360 - 8 = 1144 pt.
        let edge = Anchor {
            frame: rect(1480.0, 949.0, 30.0, 33.0),
            ..a
        };
        assert_eq!(edge.popover_origin((360.0, 480.0)).0, 2288);
    }
}
