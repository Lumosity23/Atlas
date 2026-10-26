import 'package:flutter/gestures.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:saber/atlas/multi_finger_tap.dart';

void main() {
  group('MultiFingerTapDetector', () {
    late List<int> taps;
    late MultiFingerTapDetector detector;

    setUp(() {
      taps = [];
      detector = MultiFingerTapDetector(onTap: taps.add);
    });

    Duration ms(int value) => Duration(milliseconds: value);

    void down(
      int pointer,
      Offset position,
      int at, {
      PointerDeviceKind kind = .touch,
    }) => detector.handleEvent(
      PointerDownEvent(
        pointer: pointer,
        position: position,
        timeStamp: ms(at),
        kind: kind,
      ),
    );
    void move(int pointer, Offset position, int at) => detector.handleEvent(
      PointerMoveEvent(pointer: pointer, position: position, timeStamp: ms(at)),
    );
    void up(
      int pointer,
      Offset position,
      int at, {
      PointerDeviceKind kind = .touch,
    }) => detector.handleEvent(
      PointerUpEvent(
        pointer: pointer,
        position: position,
        timeStamp: ms(at),
        kind: kind,
      ),
    );

    test('a quick two-finger tap is reported once, with two fingers', () {
      down(1, const Offset(100, 100), 0);
      down(2, const Offset(200, 100), 20);
      up(1, const Offset(100, 100), 90);
      up(2, const Offset(200, 100), 100);
      expect(taps, [2]);
    });

    test('a quick three-finger tap is reported with three fingers', () {
      down(1, const Offset(100, 100), 0);
      down(2, const Offset(200, 100), 10);
      down(3, const Offset(300, 100), 20);
      up(1, const Offset(100, 100), 80);
      up(2, const Offset(200, 100), 90);
      up(3, const Offset(300, 100), 100);
      expect(taps, [3]);
    });

    test('a single finger is never a tap', () {
      down(1, const Offset(100, 100), 0);
      up(1, const Offset(100, 100), 50);
      expect(taps, isEmpty);
    });

    test('a pinch is not a tap', () {
      down(1, const Offset(100, 100), 0);
      down(2, const Offset(200, 100), 10);
      move(1, const Offset(60, 100), 50);
      move(2, const Offset(240, 100), 50);
      up(1, const Offset(60, 100), 100);
      up(2, const Offset(240, 100), 110);
      expect(taps, isEmpty);
    });

    test('a two-finger pan is not a tap', () {
      down(1, const Offset(100, 100), 0);
      down(2, const Offset(200, 100), 10);
      move(1, const Offset(100, 160), 50);
      move(2, const Offset(200, 160), 50);
      up(1, const Offset(100, 160), 100);
      up(2, const Offset(200, 160), 110);
      expect(taps, isEmpty);
    });

    test('a tiny jitter under the travel limit is still a tap', () {
      down(1, const Offset(100, 100), 0);
      down(2, const Offset(200, 100), 10);
      move(1, const Offset(103, 101), 40);
      up(1, const Offset(103, 101), 90);
      up(2, const Offset(200, 100), 100);
      expect(taps, [2]);
    });

    test('fingers held too long are not a tap', () {
      down(1, const Offset(100, 100), 0);
      down(2, const Offset(200, 100), 10);
      up(1, const Offset(100, 100), 400);
      up(2, const Offset(200, 100), 410);
      expect(taps, isEmpty);
    });

    test(
      'a finger landing after another was lifted is not part of the tap',
      () {
        down(1, const Offset(100, 100), 0);
        down(2, const Offset(200, 100), 10);
        up(1, const Offset(100, 100), 50);
        down(3, const Offset(300, 100), 60);
        up(2, const Offset(200, 100), 90);
        up(3, const Offset(300, 100), 100);
        expect(taps, isEmpty);
      },
    );

    test('a palm resting while the stylus writes never triggers a tap', () {
      down(9, const Offset(50, 50), 0, kind: .stylus);
      down(1, const Offset(100, 300), 10);
      down(2, const Offset(140, 300), 20);
      up(1, const Offset(100, 300), 80);
      up(2, const Offset(140, 300), 90);
      up(9, const Offset(60, 50), 100, kind: .stylus);
      expect(taps, isEmpty);
    });

    test('a stylus landing during the tap cancels it', () {
      down(1, const Offset(100, 300), 0);
      down(2, const Offset(140, 300), 10);
      down(9, const Offset(50, 50), 30, kind: .stylus);
      up(1, const Offset(100, 300), 80);
      up(2, const Offset(140, 300), 90);
      expect(taps, isEmpty);
    });

    test('a cancelled pointer cancels the tap', () {
      down(1, const Offset(100, 100), 0);
      down(2, const Offset(200, 100), 10);
      detector.handleEvent(const PointerCancelEvent(pointer: 1));
      up(2, const Offset(200, 100), 100);
      expect(taps, isEmpty);
    });

    test('two taps in a row are both reported', () {
      down(1, const Offset(100, 100), 0);
      down(2, const Offset(200, 100), 10);
      up(1, const Offset(100, 100), 80);
      up(2, const Offset(200, 100), 90);
      down(3, const Offset(100, 100), 300);
      down(4, const Offset(200, 100), 310);
      up(3, const Offset(100, 100), 380);
      up(4, const Offset(200, 100), 390);
      expect(taps, [2, 2]);
    });
  });
}
