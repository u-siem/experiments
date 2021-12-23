fn main() {
    windows::build!(
        windows::Win32::system_services::{CreateEventW, SetEvent, WaitForSingleObject, HANDLE},
        windows::Win32::windows_programming::CloseHandle,
        windows::Win32::windows_event_log::{EvtSubscribe, EVT_SUBSCRIBE_FLAGS, EVT_SUBSCRIBE_NOTIFY_ACTION,EvtOpenChannelEnum,EvtNextChannelPath, EvtRender,EVT_RENDER_FLAGS},
        windows::Win32::system_services::MB_OK,
        windows::Win32::windows_and_messaging::MessageBoxA,
        windows::Win32::System::EventLog,
        windows::Win32::debug::GetLastError,
        windows::ui::notifications::{ToastNotification, ToastNotifier, ToastNotificationManager},
        windows::data::xml::dom::XmlDocument
    );
}