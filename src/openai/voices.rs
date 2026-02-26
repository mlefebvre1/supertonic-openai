use axum::Json;

// async fn create_voice() -> &'static str {
//     /* Parameters:
//      * auido_sample (file):
//      * Supported MIME types: audio/mpeg, audio/wav, audio/x-wav, audio/ogg, audio/aac, audio/flac, audio/webm, audio/mp4.
//      * could support more..
//      *
//      * consent(string): The consent recording ID
//      *
//      * name(string): Name of the voice
//      *
//      * */
//     "TODO"
// }
// async fn combine_voices() -> &'static str {
//     "TODO"
// }

pub async fn list_voices() -> Json<Vec<String>> {
    Json(vec!["F1".to_string()])
}
