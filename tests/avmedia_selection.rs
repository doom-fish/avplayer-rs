mod support;

#[test]
fn avmedia_selection_exposes_asset_and_group_surface() -> support::TestResult {
    let asset = support::loaded_audio_asset("test-avmedia-selection")?;
    let characteristics = asset.available_media_characteristics_with_media_selection_options()?;

    let selection = asset.preferred_media_selection()?;
    let _ = asset.all_media_selections()?;

    if let Some(characteristic) = characteristics.first() {
        if let Some(group) = asset.media_selection_group_for_media_characteristic(characteristic)? {
            let options = group.options()?;
            let _ = group.default_option();
            let _ = group.allows_empty_selection();

            let selected = selection.selected_media_option_in_group(&group);
            if let Some(option) = selected.as_ref().or_else(|| options.first()) {
                let _ = option.media_type()?;
                let _ = option.media_sub_types()?;
                let _ = option.is_playable()?;
                let _ = option.extended_language_tag()?;
                let _ = option.locale_identifier()?;
                let _ = option.display_name()?;
                let _ = option.common_metadata()?;
                let _ = option.available_metadata_formats()?;
                let _ = option.has_media_characteristic(characteristic)?;
                let _ = option.associated_media_selection_option_in_group(&group);
            }

            let mutable = selection.mutable_copy()?;
            mutable.select_media_option(selected.as_ref(), &group);
            assert_eq!(
                mutable.selected_media_option_in_group(&group).is_some(),
                selected.is_some()
            );
            assert_eq!(
                selection.media_selection_criteria_can_be_applied_automatically_to_group(&group),
                mutable.media_selection_criteria_can_be_applied_automatically_to_group(&group)
            );
        }
    }
    Ok(())
}
