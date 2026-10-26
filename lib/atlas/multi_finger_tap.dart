import 'package:flutter/gestures.dart';

/// Recognizes a quick tap made with several fingers at once, while the
/// canvas keeps receiving the same pointers for zooming and panning.
///
/// It only observes raw pointer events: it never enters the gesture arena,
/// so a pinch or a pan still works exactly as before. A gesture counts as a
/// tap only if every finger is lifted quickly and none of them moved; any
/// pinch or pan moves at least one finger, so it is never mistaken for a tap.
class MultiFingerTapDetector {
  new({required this.onTap});

  /// Called with the number of fingers once a valid tap ends.
  final void Function(int fingerCount) onTap;

  /// Longest time between the first finger down and the last finger up.
  static const maxTapDuration = Duration(milliseconds: 250);

  /// Furthest a finger may travel, in logical pixels, before the gesture
  /// is treated as a pinch or a pan instead.
  static const maxFingerTravel = kTouchSlop;

  final _downPositions = <int, Offset>{};
  final _activePointers = <int>{};
  final _stylusPointers = <int>{};
  Duration? _start;
  var _maxFingers = 0;
  var _aFingerWasLifted = false;
  var _invalid = false;

  void handleEvent(PointerEvent event) {
    if (event.kind != PointerDeviceKind.touch) {
      _trackStylus(event);
      return;
    }

    switch (event) {
      case PointerDownEvent():
        _onDown(event);
      case PointerMoveEvent():
        _onMove(event);
      case PointerUpEvent():
        _onUp(event);
      case PointerCancelEvent():
        _invalid = true;
        _release(event.pointer);
      default:
        break;
    }
  }

  /// A stylus touching the screen means the hand is writing: a resting palm
  /// must never undo anything.
  void _trackStylus(PointerEvent event) {
    switch (event) {
      case PointerDownEvent():
        _stylusPointers.add(event.pointer);
        if (_activePointers.isNotEmpty) _invalid = true;
      case PointerUpEvent() || PointerCancelEvent():
        _stylusPointers.remove(event.pointer);
      default:
        break;
    }
  }

  void _onDown(PointerDownEvent event) {
    if (_activePointers.isEmpty) _reset(event.timeStamp);
    // A finger landing after another was lifted is a second tap, not part
    // of this one.
    if (_aFingerWasLifted || _stylusPointers.isNotEmpty) _invalid = true;

    _activePointers.add(event.pointer);
    _downPositions[event.pointer] = event.position;
    if (_activePointers.length > _maxFingers) {
      _maxFingers = _activePointers.length;
    }
  }

  void _onMove(PointerMoveEvent event) {
    final downPosition = _downPositions[event.pointer];
    if (downPosition == null) return;
    if ((event.position - downPosition).distance > maxFingerTravel) {
      _invalid = true;
    }
  }

  void _onUp(PointerUpEvent event) {
    if (!_activePointers.contains(event.pointer)) return;
    _aFingerWasLifted = true;
    _release(event.pointer);
    if (_activePointers.isNotEmpty) return;

    final start = _start;
    final isQuick = start != null && event.timeStamp - start <= maxTapDuration;
    if (!_invalid && isQuick && _maxFingers >= 2) onTap(_maxFingers);
  }

  void _release(int pointer) {
    _activePointers.remove(pointer);
    _downPositions.remove(pointer);
  }

  void _reset(Duration start) {
    _start = start;
    _maxFingers = 0;
    _aFingerWasLifted = false;
    _invalid = false;
  }
}
