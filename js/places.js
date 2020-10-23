export const init_autocomplete_js = (id, callback) => {
    // Create the autocomplete object, restricting the search predictions to
    // geographical location types.
    const autocomplete = new google.maps.places.Autocomplete(
        document.getElementById(id),
        { types: ["geocode"] }
    );
    // Avoid paying for data that you don't need by restricting the set of
    // place fields that are returned to just the address components.
    autocomplete.setFields(["name", "geometry"]);
    // When the user selects an address from the drop-down, populate the
    // address fields in the form.
    autocomplete.addListener("place_changed", () => {
        const place = autocomplete.getPlace();
        callback(JSON.stringify(place));
    });
}
