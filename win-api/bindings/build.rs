fn main() {
    windows::build!(
        windows::win32::system_services::{CreateEventW, SetEvent, WaitForSingleObject, HANDLE},
        windows::win32::windows_programming::CloseHandle,
        windows::win32::windows_event_log::{EvtSubscribe, EVT_SUBSCRIBE_FLAGS, EVT_SUBSCRIBE_NOTIFY_ACTION,EvtOpenChannelEnum,EvtNextChannelPath, EvtRender,EVT_RENDER_FLAGS},
        windows::win32::system_services::MB_OK,
        windows::win32::windows_and_messaging::MessageBoxA,
        windows::win32::debug::GetLastError,
        windows::ui::notifications::{ToastNotification, ToastNotifier, ToastNotificationManager},
        windows::data::xml::dom::XmlDocument
    );
}