import 'package:flutter/gestures.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_screenshot/golden_screenshot.dart';
import 'package:saber/components/canvas/interactive_canvas.dart';
import 'package:saber/data/file_manager/file_manager.dart';
import 'package:saber/data/flavor_config.dart';
import 'package:saber/data/prefs.dart';
import 'package:saber/i18n/strings.g.dart';
import 'package:saber/pages/editor/editor.dart';

import 'editor_undo_redo_test.dart' show drawOnEditor;

import 'utils/test_mock_channel_handlers.dart';

void main() {
  Future<EditorState> openEditor(WidgetTester tester) async {
    TestWidgetsFlutterBinding.ensureInitialized();
    setupMockPathProvider();
    setupMockPrinting();
    FlavorConfig.setup();
    await tester.runAsync(FileManager.init);

    await tester.pumpWidget(
      TranslationProvider(
        child: ScreenshotApp(
          device: GoldenScreenshotDevices.androidPhone.device,
          home: Editor(),
        ),
      ),
    );

    final editorState = tester.state<EditorState>(find.byType(Editor));
    addTearDown(editorState.cancelAutosaveAndMarkSaved);
    // A global pref: touch() turns it off, so each test starts from the
    // default where fingers draw the initial strokes.
    stows.editorFingerDrawing.value = true;
    return editorState;
  }

  int strokeCount(EditorState editor) =>
      editor.coreInfo.pages.first.strokes.length;

  double canvasScale(WidgetTester tester) => tester
      .widget<InteractiveCanvasViewer>(find.byType(InteractiveCanvasViewer))
      .transformationController!
      .value
      .getMaxScaleOnAxis();

  /// Touches the canvas with [offsets.length] fingers, moves each finger
  /// by its [moves] entry, then lifts them all.
  Future<void> touch(
    WidgetTester tester,
    List<Offset> offsets, {
    List<Offset>? moves,
  }) async {
    // Writing with a stylus turns finger drawing off
    // (autoDisableFingerDrawingWhenStylusDetected): fingers only navigate.
    stows.editorFingerDrawing.value = false;
    final center = tester.getCenter(find.byType(Editor));
    final fingers = <TestGesture>[];
    for (var i = 0; i < offsets.length; i++) {
      fingers.add(
        await tester.startGesture(
          center + offsets[i],
          pointer: 100 + i,
          kind: PointerDeviceKind.touch,
        ),
      );
    }
    await tester.pump(const Duration(milliseconds: 10));
    if (moves != null) {
      for (var step = 1; step <= 5; step++) {
        for (var i = 0; i < fingers.length; i++) {
          await fingers[i].moveBy(moves[i] / 5);
        }
        await tester.pump(const Duration(milliseconds: 10));
      }
    }
    for (final finger in fingers) {
      await finger.up();
    }
    await tester.pumpAndSettle();
    // Lets the editor's pointer-count reset timer fire, and after a zoom or
    // a pan, the zoom HUD's 5 s hide timer.
    await tester.pump(
      moves == null
          ? const Duration(milliseconds: 50)
          : const Duration(seconds: 6),
    );
  }

  const twoFingers = [Offset(-40, 0), Offset(40, 0)];
  const threeFingers = [Offset(-60, 0), Offset.zero, Offset(60, 0)];

  testGoldens('Atlas: a two-finger tap undoes exactly one action', (
    tester,
  ) async {
    final editor = await openEditor(tester);
    await drawOnEditor(tester);
    await tester.pump();
    await drawOnEditor(tester);
    await tester.pump();
    expect(strokeCount(editor), 2);

    await touch(tester, twoFingers);
    expect(strokeCount(editor), 1);
  });

  testGoldens('Atlas: a three-finger tap redoes what was undone', (
    tester,
  ) async {
    final editor = await openEditor(tester);
    await drawOnEditor(tester);
    await tester.pump();
    await touch(tester, twoFingers);
    expect(strokeCount(editor), 0);

    await touch(tester, threeFingers);
    expect(strokeCount(editor), 1);
  });

  testGoldens('Atlas: a two-finger tap leaves no stroke on the page', (
    tester,
  ) async {
    final editor = await openEditor(tester);
    await touch(tester, twoFingers);
    expect(strokeCount(editor), 0);
    expect(tester.takeException(), isNull);
  });

  testGoldens('Atlas: a pinch still zooms and undoes nothing', (tester) async {
    final editor = await openEditor(tester);
    await drawOnEditor(tester);
    await tester.pump();
    final scaleBefore = canvasScale(tester);

    await touch(
      tester,
      twoFingers,
      moves: const [Offset(-60, 0), Offset(60, 0)],
    );
    expect(strokeCount(editor), 1);
    expect(canvasScale(tester), greaterThan(scaleBefore));
  });

  testGoldens('Atlas: a two-finger pan undoes nothing', (tester) async {
    final editor = await openEditor(tester);
    await drawOnEditor(tester);
    await tester.pump();

    await touch(
      tester,
      twoFingers,
      moves: const [Offset(0, -80), Offset(0, -80)],
    );
    expect(strokeCount(editor), 1);
  });
}
