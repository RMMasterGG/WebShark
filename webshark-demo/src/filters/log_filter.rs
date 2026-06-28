use std::time::Instant;
use webshark::bytes::Bytes;
use webshark::tracing::{error, info, info_span};
use webshark::{Request, Response};
use webshark::auth::authentication::{Filter, FilterContext};
use webshark::utils::other::BoxFuture;

pub struct LoggerFilter;

impl Filter for LoggerFilter {
    fn do_filter<'a>(
        &self,
        request: Request<Bytes>,
        context: &'a mut FilterContext,
    ) -> BoxFuture<'a, Result<Response<Bytes>, &'static str>> {
        // Упаковываем всю логику в асинхронный BoxFuture
        Box::pin(async move {
            // Безопасно вытаскиваем строковые значения
            let method_str = request.method().as_str().to_string();
            let path_str = request.uri().path().to_string();

            // Создаем span для красивых логов tracing
            let auth_span = info_span!(
                "http_request",
                method = %method_str,
                path = %path_str
            );

            // Входим в контекст спана
            let _guard = auth_span.enter();

            info!("Начало обработки запроса");
            let start_time = Instant::now();

            // ИСПРАВЛЕНО: Передаем запрос дальше по асинпочке без лишнего handler и через .await!
            let response_result = context.next_filter(request).await;

            let duration = start_time.elapsed();
            let duration_ms = duration.as_secs_f64() * 1000.0;

            match &response_result {
                Ok(response) => {
                    info!(
                        status = ?response.get_status(),
                        duration_ms = %duration_ms,
                        "Запрос успешно обработан"
                    );
                }
                Err(err_msg) => {
                    error!(
                        error = %err_msg,
                        duration_ms = %duration_ms,
                        "Ошибка при обработке запроса"
                    );
                }
            }

            response_result
        })
    }
}
