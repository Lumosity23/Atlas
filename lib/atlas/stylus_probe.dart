import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// Built only with `--dart-define=ATLAS_STYLUS_PROBE=true`.
///
/// Shows which event the tablet sends when the stylus is double-tapped, so
/// it can be wired to the eraser. Only event kinds and key identifiers are
/// shown, never note content, and nothing is logged.
const stylusProbeEnabled = bool.fromEnvironment('ATLAS_STYLUS_PROBE');

class StylusProbe extends StatefulWidget {
  const new({super.key, required this.child});

  final Widget child;

  @override
  State<StylusProbe> createState() => _StylusProbeState();
}

class _StylusProbeState extends State<StylusProbe> {
  static const _maxLines = 8;
  final _lines = <String>[];

  @override
  void initState() {
    super.initState();
    HardwareKeyboard.instance.addHandler(_onKey);
  }

  @override
  void dispose() {
    HardwareKeyboard.instance.removeHandler(_onKey);
    super.dispose();
  }

  bool _onKey(KeyEvent event) {
    _show(
      'key ${event.runtimeType}: logical=${event.logicalKey.debugName} '
      '(0x${event.logicalKey.keyId.toRadixString(16)}), '
      'physical=0x${event.physicalKey.usbHidUsage.toRadixString(16)}',
    );
    return false;
  }

  void _onPointer(PointerEvent event) {
    // Moves are too frequent to be useful here.
    if (event is PointerMoveEvent || event is PointerHoverEvent) return;
    _show(
      'pointer ${event.runtimeType}: kind=${event.kind.name}, '
      'buttons=0x${event.buttons.toRadixString(16)}',
    );
  }

  void _show(String line) {
    setState(() {
      _lines.insert(0, line);
      if (_lines.length > _maxLines) _lines.removeLast();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Listener(
      onPointerDown: _onPointer,
      onPointerUp: _onPointer,
      onPointerSignal: _onPointer,
      behavior: HitTestBehavior.translucent,
      // Sits above the app, outside of its Directionality.
      child: Directionality(
        textDirection: TextDirection.ltr,
        child: Stack(
          children: [
            widget.child,
            Positioned(
              left: 8,
              right: 8,
              bottom: 8,
              child: IgnorePointer(
                child: Container(
                  padding: const EdgeInsets.all(8),
                  color: const Color(0xCC000000),
                  child: Text(
                    _lines.isEmpty ? 'Atlas stylus probe' : _lines.join('\n'),
                    style: const TextStyle(
                      color: Color(0xFFFFFFFF),
                      fontSize: 12,
                      fontFamily: 'monospace',
                    ),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
