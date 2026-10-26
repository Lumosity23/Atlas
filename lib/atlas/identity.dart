/// Name of the local notes folder, distinct from Saber's `Saber` folder.
///
/// Saber's `appRootDirectoryPrefix` also names the remote sync folder, so it
/// stays untouched; only the local folder changes.
const atlasNotesFolder = 'Atlas';

/// Saber's Nextcloud sync stays off until Atlas replaces it with the Pensay
/// API: logged into the same account, the fork would write into the same
/// remote files as Saber.
const atlasNextcloudSyncEnabled = false;
