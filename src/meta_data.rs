#[derive(Clone, Debug)]
pub struct TrackData {
    pub order: usize,
    pub name: String,
    pub start: usize, // In seconds
    pub end: Option<usize>, // If None, the end is the end of the song
}

pub fn parse_track_data(source: &str) -> Result<Vec<TrackData>, String> {
    let mut start_points_and_names: Vec<(usize, String)> = Vec::new();

    for line in source.lines().filter(|v| v.trim().len() != 0) {
        let mut sections = line.split_whitespace().filter(|v| v.trim().len() != 0);

        // The first section should be a timestamp
        let mut time_stamp = sections.next().ok_or_else(|| format!("A timestamp is required on each line"))?;
        let mut seconds: usize = 0;

        // We are doing this to support '2:23' formatting, but also '1:5:04' formatting, i.e. hours
        // and minutes.
        for section in time_stamp.split(':') {
            seconds *= 60;
            seconds += section.parse::<usize>().map_err(|e| format!("{:?}", e))?;
        }

        let mut name = String::new();
        for (i, section) in sections.enumerate() {
            if i > 0 {
                name.push_str(" ");
            }
            name.push_str(section);
        }

        start_points_and_names.push((seconds, name));
    }

    let mut tracks = Vec::new();
    for (i, (start_point, name)) in start_points_and_names.iter().enumerate() {
        let start = start_point;
        let end = if i < start_points_and_names.len() - 1 {
            Some(start_points_and_names[i + 1].0)
        }else{
            None
        };

        tracks.push(TrackData {
            order: i,
            name: name.clone(),
            start: *start,
            end: end
        });
    }

    Ok(tracks)
}
