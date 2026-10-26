import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:saber/atlas/identity.dart';
import 'package:saber/data/file_manager/file_manager.dart';

import 'utils/test_mock_channel_handlers.dart';

/// Installing, reinstalling or removing Atlas must never touch the user's
/// Saber install or its notes: every identifier below must differ.
void main() {
  const saberId = 'com.adilhanney.saber';

  String read(String path) => File(path).readAsStringSync();

  test('the Android application id is not Saber\'s', () {
    final gradle = read('android/app/build.gradle.kts');
    final applicationId = RegExp(r'applicationId = "([^"]+)"')
        .firstMatch(gradle)!
        .group(1);
    expect(applicationId, isNot(saberId));
  });

  test('the clipboard provider authority follows the application id', () {
    // Two apps declaring the same authority cannot both be installed.
    final manifest = read('android/app/src/main/AndroidManifest.xml');
    expect(manifest, isNot(contains('"$saberId.SuperClipboardDataProvider"')));
    expect(
      manifest,
      contains(
        r'android:authorities="${applicationId}.SuperClipboardDataProvider"',
      ),
    );
  });

  test('the Linux application id is not Saber\'s', () {
    final cmake = read('linux/CMakeLists.txt');
    expect(cmake, isNot(contains('set(APPLICATION_ID "$saberId")')));
  });

  test('the Windows product name is not Saber', () {
    // Settings on Windows live in a folder named after the company and
    // product names.
    final resources = read('windows/runner/Runner.rc');
    expect(resources, isNot(contains('VALUE "ProductName", "Saber"')));
    expect(resources, isNot(contains('VALUE "CompanyName", "com.adilhanney"')));
  });

  test('notes live in their own folder, not in Saber\'s', () async {
    TestWidgetsFlutterBinding.ensureInitialized();
    setupMockPathProvider();
    final directory = await FileManager.getDefaultDocumentsDirectory();
    expect(directory, endsWith('/$atlasNotesFolder'));
    expect(
      directory,
      isNot(endsWith('/${FileManager.appRootDirectoryPrefix}')),
    );
  });

  test('Nextcloud sync is off', () {
    expect(atlasNextcloudSyncEnabled, isFalse);
  });
}
