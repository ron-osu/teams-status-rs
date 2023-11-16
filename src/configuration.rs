use crate::ha_configuration::{
    create_ha_configuration, HaConfiguration, HA_ENTITIES, HA_ICONS, HA_IN_A_MEETING,
    HA_LONG_LIVE_TOKEN, HA_MEETING_FRIENDLY_NAME, HA_MEETING_ID, HA_NOT_IN_A_MEETING, HA_URL,
    HA_VIDEO_FRIENDLY_NAME, HA_VIDEO_ID, HA_VIDEO_OFF, HA_VIDEO_ON, HOME_ASSISTANT,
};
use crate::teams_configuration::{
    create_teams_configuration, TeamsConfiguration, TEAMS, TEAMS_API_TOKEN, TEAMS_URL,
};
use crate::utils::{decrypt_if_needed, encrypt};
use ini::Ini;
use log::info;

pub struct Configuration {
    pub ha: HaConfiguration,
    pub teams: TeamsConfiguration,
}

pub fn get_configuration() -> Configuration {
    let mut conf = create_configuration();
    load_configuration(&mut conf);
    // We recreate the file each time in case we introduce new values or configs
    save_ha_configuration(&conf);
    conf
}

fn load_configuration(conf: &mut Configuration) {
    let i = Ini::load_from_file("conf.ini").unwrap_or_else(|err| {
        info!(
            "The file conf.ini could not be loaded, we will create a new one: {}",
            err.to_string()
        );
        return Ini::new();
    });

    for (sec, prop) in i.iter() {
        for (k, v) in prop.iter() {
            if v.is_empty() {
                continue;
            }

            match sec {
                Some(HOME_ASSISTANT) => match k {
                    HA_LONG_LIVE_TOKEN => conf.ha.long_live_token = decrypt_if_needed(v),
                    HA_URL => conf.ha.url = v.to_string(),
                    _ => { /* We just ignore incorrect configs */ }
                },
                Some(HA_ICONS) => match k {
                    HA_IN_A_MEETING => conf.ha.icons.in_a_meeting = v.to_string(),
                    HA_NOT_IN_A_MEETING => conf.ha.icons.not_in_a_meeting = v.to_string(),
                    HA_VIDEO_ON => conf.ha.icons.video_on = v.to_string(),
                    HA_VIDEO_OFF => conf.ha.icons.video_off = v.to_string(),
                    _ => { /* We just ignore incorrect configs */ }
                },
                Some(HA_ENTITIES) => match k {
                    HA_MEETING_ID => conf.ha.entities.meeting_id = v.to_string(),
                    HA_MEETING_FRIENDLY_NAME => {
                        conf.ha.entities.meeting_friendly_name = v.to_string()
                    }
                    HA_VIDEO_ID => conf.ha.entities.video_id = v.to_string(),
                    HA_VIDEO_FRIENDLY_NAME => conf.ha.entities.video_friendly_name = v.to_string(),
                    _ => { /* We just ignore incorrect configs */ }
                },
                Some(TEAMS) => match k {
                    TEAMS_URL => conf.teams.url = v.to_string(),
                    TEAMS_API_TOKEN => conf.teams.api_token = decrypt_if_needed(v),
                    _ => { /* We just ignore incorrect configs */ }
                },
                _ => { /* We just ignore incorrect configs */ }
            }
        }
    }
}

fn create_configuration() -> Configuration {
    Configuration {
        ha: create_ha_configuration(),
        teams: create_teams_configuration(),
    }
}
fn save_ha_configuration(conf: &Configuration) {
    let mut ini = Ini::new();
    ini.with_section(Some(TEAMS))
        .set(TEAMS_URL, &conf.teams.url)
        .set(TEAMS_API_TOKEN, encrypt(&conf.teams.api_token));
    ini.with_section(Some(HOME_ASSISTANT))
        .set(HA_URL, &conf.ha.url)
        .set(HA_LONG_LIVE_TOKEN, encrypt(&conf.ha.long_live_token));
    ini.with_section(Some(HA_ICONS))
        .set(HA_IN_A_MEETING, &conf.ha.icons.in_a_meeting)
        .set(HA_NOT_IN_A_MEETING, &conf.ha.icons.not_in_a_meeting);
    ini.with_section(Some(HA_ENTITIES))
        .set(HA_MEETING_ID, &conf.ha.entities.meeting_id)
        .set(
            HA_MEETING_FRIENDLY_NAME,
            &conf.ha.entities.meeting_friendly_name,
        )
        .set(HA_VIDEO_ID, &conf.ha.entities.video_id)
        .set(
            HA_VIDEO_FRIENDLY_NAME,
            &conf.ha.entities.video_friendly_name,
        );
    ini.write_to_file("conf.ini").unwrap();
}
