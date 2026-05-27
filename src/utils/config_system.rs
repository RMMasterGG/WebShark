//! Модуль управления конфигурацией приложения.
//!
//! Данный модуль отвечает за инициализацию, валидацию, безопасное чтение и
//! автоматическое восстановление настроек приложения из формата TOML.
//! Все считанные настройки сохраняются в потокобезопасное глобальное хранилище.

use serde_with::OneOrMany;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::OnceLock;
use http::Method;
use serde_with::serde_as;
use tracing::{error, warn};

/// Глобальный статический контейнер для хранения конфигурации приложения.
///
/// Инициализируется ровно один раз при старте программы с помощью метода [`Config::init_config`].
/// Предоставляет потокобезопасный доступ к настройкам в режиме "только чтение" из любой точки крейта.
pub static APP_CONFIG: OnceLock<Config> = OnceLock::new();

/// Настройки сетевого HTTP-сервера.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    /// IP-адрес или хост, на котором разворачивается сервер (например, `"127.0.0.1"`).
    address: String,
    /// Сетевой порт, который будет слушать приложение (например, `8080`).
    port: u16,
}

impl ServerConfig {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Вспомогательный метод, собирающий адрес и порт сервера в единую строку формата `address:port`.
    ///
    /// Используется для передачи в метод связывания сетевого сокета (например, `TcpListener::bind`).
    pub fn server_and_port(&self) -> String {
        format!("{}:{}", &self.address, &self.port)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".into(),
            port: 8080,
        }
    }
}

/// Настройки подключения к базе данных.
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// URL или имя строки подключения (например, `"AutoShopService"`).
    pub url: String,
    /// Имя пользователя для авторизации в СУБД.
    pub user: String,
    /// Пароль для доступа к базе данных.
    pub password: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "AutoShopService".into(),
            user: "user".into(),
            password: "password".into()
        }
    }
}

/// Настройки политики CORS (Cross-Origin Resource Sharing).
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Разрешенные домены, например: `vec!["http://localhost:3000".into()]`
    #[serde_as(deserialize_as = "OneOrMany<_>")]
    pub allowed_origins: Vec<String>,
    /// Разрешенные методы, например: `vec!["GET".into(), "POST".into()]`
    #[serde_as(deserialize_as = "OneOrMany<_>")]
    pub allowed_methods: Vec<String>,
    /// Разрешенные заголовки, например: `vec!["Content-Type".into()]`
    #[serde_as(deserialize_as = "OneOrMany<_>")]
    pub allowed_headers: Vec<String>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!("*".into()),
            allowed_methods: vec!("*".into()),
            allowed_headers: vec!("*".into()),
        }
    }
}

impl CorsConfig {
    pub fn allowed_origins(&self) -> Vec<String> {
        self.allowed_origins.clone()
    }

    pub fn allowed_methods(&self) -> Vec<Method> {
        self.allowed_methods
            .clone()
            .into_iter()
            .filter_map(|m| Method::from_bytes(m.as_bytes()).ok())
            .collect()
    }

    pub fn allowed_headers(&self) -> Vec<String> {
        self.allowed_headers.clone()
    }

    pub fn is_allowed_origin(&self, origin: &str) -> bool {
        self.allowed_origins.iter().any(|o| o == "*" || o == origin)
    }

    pub fn is_allowed_method(&self, method: Method) -> bool {
        let method_str = method.as_str();
        self.allowed_methods.iter().any(|m| m == "*" || m == method_str)
    }

    pub fn is_allowed_header(&self, header: &str) -> bool {
        self.allowed_headers.iter().any(|h| h == "*" || h == header)
    }
}

/// Настройки подсистемы логирования приложения.
#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Глобальный уровень логирования (например: `"Info"`, `"Debug"`, `"Error"`).
    pub level: String,
    /// Шаблон форматирования вывода логов в консоль или файл.
    pub pattern: String,
    /// Путь к файлу на диске, куда будут записываться логи (например, `"logs/app.log"`).
    pub file: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "Info".into(),
            pattern: "%d{yyyy-MM-dd HH:mm:ss} - %highlight(%-5level) [%thread] %cyan(%logger{36}) - %msg%n".into(),
            file: "logs/app.log".into(),
        }
    }
}

impl LoggingConfig {
    pub fn level(&self) -> String {
        self.level.clone()
    }
    pub fn pattern(&self) -> String {
        self.pattern.clone()
    }
    pub fn file(&self) -> String {
        self.file.clone()
    }
}

/// Главная структура конфигурации, объединяющая все подсистемы приложения.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Секция конфигурации сервера.
    server: ServerConfig,
    /// Секция конфигурации базы данных.
    database: DatabaseConfig,
    /// Секция конфигурации политик CORS.
    cors: CorsConfig,
    /// Секция конфигурации логирования.
    logging: LoggingConfig,
}

impl Config {
    /// Возвращает ссылку на настройки сервера.
    pub fn server(&self) -> &ServerConfig {
        &self.server
    }

    /// Возвращает ссылку на настройки базы данных.
    pub fn database(&self) -> &DatabaseConfig {
        &self.database
    }

    /// Возвращает ссылку на настройки CORS.
    pub fn cors(&self) -> &CorsConfig {
        &self.cors
    }

    /// Возвращает ссылку на настройки логирования.
    pub fn logging(&self) -> &LoggingConfig {
        &self.logging
    }
}

impl Default for Config {
    /// Задает эталонные настройки по умолчанию, которые гарантируют корректный запуск приложения.
    fn default() -> Self {
        Self {
            server: Default::default(),
            database: DatabaseConfig::default(),
            cors: Default::default(),
            logging: Default::default(),
        }
    }
}

impl Config {
    /// Инициализирует глобальную конфигурацию приложения.
    ///
    /// # Алгоритм работы:
    /// 1. Проверяет наличие файла `config.toml` по пути `./src/resources/`.
    /// 2. Если файл отсутствует, метод создает дерево папок, генерирует дефолтный файл
    ///    на базе [`Config::default`] с красивым форматированием и возвращает его.
    /// 3. Если файл существует, считывает его контент и десериализует.
    /// 4. Сохраняет итоговую структуру в глобальный статический [`APP_CONFIG`].
    ///
    /// # Паника
    /// Метод вызовет панику (`panic!`), если существующий файл конфигурации поврежден,
    /// имеет некорректный синтаксис TOML или не совпадает со структурой `Config`.
    pub(crate) fn init_config() {
        let path = "./src/resources/config.toml";

        let config = match fs::read_to_string(path) {
            // Файл успешно прочитан с диска
            Ok(content) => match toml::from_str::<Config>(&content) {
                Ok(parsed_cfg) => parsed_cfg,
                Err(err) => {
                    // Используем многострочную запись в tracing, чтобы не плодить рамки вручную
                    error!(
                    target: "config_loader",
                    "КРИТИЧЕСКАЯ ОШИБКА: Файл конфигурации '{}' поврежден!\n\
                     Детали ошибки: {}\n\
                     Приложение остановлено. Требуется восстановление.",
                    path, err
                );

                    // В точке инициализации паника — это норма. Без конфига сервер не поднять.
                    panic!("Configuration file corrupted: {}", err);
                }
            },
            // Файла не существует (первый запуск проекта)
            Err(_) => {
                let default_cfg = Config::default();

                if let Err(e) = fs::create_dir_all("./src/resources") {
                    error!("Не удалось создать директорию для конфигурации: {}", e);
                }

                match toml::to_string_pretty(&default_cfg) {
                    Ok(toml_default) => {
                        if let Err(e) = fs::write(path, toml_default) {
                            error!("Не удалось записать дефолтный конфиг: {}", e);
                        } else {
                            warn!("Создан дефолтный файл конфигурации по пути: {}", path);
                        }
                    }
                    Err(e) => error!("Ошибка сериализации дефолтного конфига: {}", e),
                }

                default_cfg
            }
        };

        if APP_CONFIG.set(config).is_err() {
            error!("Попытка повторной инициализации глоабального конфига APP_CONFIG!");
        }
    }

}
