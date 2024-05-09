use anyhow::Result;

use crate::mtga_events::client::RequestTypeClientToMatchServiceMessage;
use crate::mtga_events::gre::RequestTypeGREToClientEvent;
use crate::mtga_events::mgrsc::RequestTypeMGRSCEvent;

#[derive(Debug)]
pub enum ParseOutput {
    GREMessage(RequestTypeGREToClientEvent),
    ClientMessage(RequestTypeClientToMatchServiceMessage),
    MGRSCMessage(RequestTypeMGRSCEvent),
    NoEvent,
}


pub fn parse(event: &str) -> Result<ParseOutput> {
    if event.contains("clientToMatchServiceMessage") {
        let client_to_match_service_message: RequestTypeClientToMatchServiceMessage =
            serde_json::from_str(event)?;
        Ok(ParseOutput::ClientMessage(client_to_match_service_message))
    } else if event.contains("matchGameRoomStateChangedEvent") {
        let mgrsc_event: RequestTypeMGRSCEvent = serde_json::from_str(event)?;
        Ok(ParseOutput::MGRSCMessage(mgrsc_event))
    } else if event.contains("greToClientEvent") {
        let request_gre_to_client_event: RequestTypeGREToClientEvent =
            serde_json::from_str(event)?;
        Ok(ParseOutput::GREMessage(request_gre_to_client_event))
    } else {
        Ok(ParseOutput::NoEvent)
    }
}

