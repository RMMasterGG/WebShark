pub mod controller;

use std::io::stdout;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::prelude::{Modifier, Span};
use ratatui::style::{Color, Style};
use ratatui::Terminal;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

pub fn command_generate(args: Vec<String>) {
    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // 1. Массив ваших компонентов для генерации
    let items = vec![
        " component (c)    A generic web component / controller",
        " route (r)        A new endpoint route handler",
        " dto (d)          A data transfer object validation model",
    ];

    // Индекс текущего выбранного пункта
    let mut selected_index = 0;

    // Очищаем буфер ввода перед стартом (чтобы старый Enter из консоли не закрыл окно)
    while event::poll(std::time::Duration::from_millis(0)).unwrap() {
        let _ = event::read().unwrap();
    }

    loop {
        terminal.draw(|f| {
            // 1. Берем РЕАЛЬНЫЙ размер окна терминала пользователя
            let size = f.area();

            // 2. Рассчитываем ширину с умом:
            // Если экран огромный (больше 84 символов), делаем рамку фиксированной на 80 символов.
            // Если экран маленький, сжимаем рамку под размер экрана, оставляя отступы по 2 символа с боков.
            let width = if size.width > 84 { 80 } else { size.width.saturating_sub(4) };

            // Высоту тоже можно сделать адаптивной или зафиксировать под количество пунктов меню (10 строк)
            let height = if size.height > 12 { 10 } else { size.height.saturating_sub(2) };

            // 3. Создаем область для рисования
            let area = Rect::new(2, 1, width, height);

            let claude_block = Block::default()
                .title(Span::styled(" WebShark Generator ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Red));

            // 2. Формируем строки текста динамически
            let mut lines = vec![
                ratatui::text::Line::from(Span::styled("Select component type:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
                ratatui::text::Line::from(""), // Пустая строка-отступ
            ];

            for (i, item) in items.iter().enumerate() {
                if i == selected_index {
                    // Подсвечиваем выбранный пункт зеленым цветом и добавляем маркер ">"
                    let styled_item = format!(" > {}", &item[1..]); // Заменяем первый пробел на стрелочку
                    lines.push(ratatui::text::Line::from(Span::styled(styled_item, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))));
                } else {
                    // Обычные пункты выводим стандартным белым/серым цветом
                    lines.push(ratatui::text::Line::from(Span::styled(*item, Style::default().fg(Color::Gray))));
                }
            }

            let paragraph = Paragraph::new(lines)
                .block(claude_block);

            f.render_widget(paragraph, area);
        }).unwrap();

        // 3. Обработка нажатий клавиш управления
        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                // Стрелочка ВВЕРХ
                KeyCode::Up => {
                    if selected_index > 0 {
                        selected_index -= 1;
                    } else {
                        selected_index = items.len() - 1; // Цикличный скролл на самый низ
                    }
                }
                // Стрелочка ВНИЗ
                KeyCode::Down => {
                    if selected_index < items.len() - 1 {
                        selected_index += 1;
                    } else {
                        selected_index = 0; // Цикличный скролл на самый верх
                    }
                }
                // Нажатие ENTER (подтверждение выбора)
                KeyCode::Enter => {
                    // Выходим из графического режима, чтобы обработать выбор
                    break;
                }
                // Жесткий выход на Esc или 'q'
                KeyCode::Char('q') | KeyCode::Esc => {
                    disable_raw_mode().unwrap();
                    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
                    terminal.show_cursor().unwrap();
                    return; // Полностью выходим из функции генерации
                }
                _ => {}
            }
        }
    }

    // Закрываем графический экран после нажатия Enter
    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    terminal.show_cursor().unwrap();

    // 4. Логика после выбора пункта меню
    match selected_index {
        0 => println!("Вы выбрали генерацию компонента (component)!"),
        1 => println!("Вы выбрали генерацию маршрута (route)!"),
        2 => println!("Вы выбрали генерацию DTO!"),
        _ => {}
    }
    // Здесь дальше вызывается ваша функция std::fs::write для генерации кода
}
