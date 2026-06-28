use webshark::bytes::Bytes;
use webshark::{Request, Response};
use webshark::auth::authentication::{Filter, FilterContext};
use webshark::utils::other::BoxFuture;

pub struct AuthFilter;

impl Filter for AuthFilter {
    fn do_filter<'a>(
        &self,
        request: Request<Bytes>,
        context: &'a mut FilterContext,
    ) -> BoxFuture<'a, Result<Response<Bytes>, &'static str>> {
        Box::pin(async move {

            // Тут в будущем будет твоя асинхронная проверка токенов, сессий или Redis через .await!

            // ИСПРАВЛЕНО: Передаем запрос дальше асинхронно без лишнего аргумента handler
            let response_result = context.next_filter(request).await;

            response_result
        })
    }
}
