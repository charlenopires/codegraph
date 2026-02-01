//! Template engine - base templates for UI components by category

use std::collections::HashMap;

/// Template engine providing base templates by category
pub struct TemplateEngine {
    templates: HashMap<String, Template>,
}

/// A UI component template
#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub category: String,
    pub html: String,
    pub css: String,
    pub javascript: Option<String>,
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            templates: HashMap::new(),
        };
        engine.load_default_templates();
        engine
    }

    /// Get template by category
    pub fn get_by_category(&self, category: &str) -> Option<&Template> {
        self.templates.get(category)
    }

    /// Get all templates
    pub fn all(&self) -> impl Iterator<Item = &Template> {
        self.templates.values()
    }

    /// Get templates matching categories
    pub fn get_matching(&self, categories: &[&str]) -> Vec<&Template> {
        categories
            .iter()
            .filter_map(|cat| self.templates.get(*cat))
            .collect()
    }

    fn load_default_templates(&mut self) {
        // Button template
        self.templates.insert(
            "button".to_string(),
            Template {
                name: "Button".to_string(),
                category: "button".to_string(),
                html: r#"<button class="btn btn--primary" type="button" aria-label="Action button">
  <span class="btn__text">Click me</span>
</button>"#
                    .to_string(),
                css: r#".btn {
  --btn-bg: var(--color-primary, #3b82f6);
  --btn-text: var(--color-white, #ffffff);
  --btn-padding: 0.75rem 1.5rem;
  --btn-radius: 0.5rem;

  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: var(--btn-padding);
  background-color: var(--btn-bg);
  color: var(--btn-text);
  border: none;
  border-radius: var(--btn-radius);
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
}

.btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.btn:focus {
  outline: 2px solid var(--btn-bg);
  outline-offset: 2px;
}

.btn:active {
  transform: translateY(0);
}"#
                    .to_string(),
                javascript: None,
            },
        );

        // Form template
        self.templates.insert(
            "form".to_string(),
            Template {
                name: "Form".to_string(),
                category: "form".to_string(),
                html: r#"<form class="form" aria-labelledby="form-title">
  <h2 id="form-title" class="form__title">Contact Form</h2>

  <div class="form__group">
    <label for="name" class="form__label">Name</label>
    <input type="text" id="name" name="name" class="form__input" required aria-required="true">
  </div>

  <div class="form__group">
    <label for="email" class="form__label">Email</label>
    <input type="email" id="email" name="email" class="form__input" required aria-required="true">
  </div>

  <div class="form__group">
    <label for="message" class="form__label">Message</label>
    <textarea id="message" name="message" class="form__textarea" rows="4" required aria-required="true"></textarea>
  </div>

  <button type="submit" class="form__submit">Send Message</button>
</form>"#
                    .to_string(),
                css: r#".form {
  --form-gap: 1.5rem;
  --form-radius: 0.5rem;

  display: flex;
  flex-direction: column;
  gap: var(--form-gap);
  max-width: 32rem;
  padding: 2rem;
  background: var(--color-surface, #ffffff);
  border-radius: var(--form-radius);
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.form__title {
  margin: 0 0 0.5rem;
  font-size: 1.5rem;
  color: var(--color-text, #1f2937);
}

.form__group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.form__label {
  font-weight: 500;
  color: var(--color-text, #374151);
}

.form__input,
.form__textarea {
  padding: 0.75rem 1rem;
  border: 1px solid var(--color-border, #d1d5db);
  border-radius: var(--form-radius);
  font-size: 1rem;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.form__input:focus,
.form__textarea:focus {
  outline: none;
  border-color: var(--color-primary, #3b82f6);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.form__submit {
  padding: 0.75rem 1.5rem;
  background: var(--color-primary, #3b82f6);
  color: white;
  border: none;
  border-radius: var(--form-radius);
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
}

.form__submit:hover {
  background: var(--color-primary-dark, #2563eb);
}"#
                    .to_string(),
                javascript: Some(
                    r#"document.querySelector('.form').addEventListener('submit', (e) => {
  e.preventDefault();
  const formData = new FormData(e.target);
  const data = Object.fromEntries(formData);
  console.log('Form submitted:', data);
  // Add your form submission logic here
});"#
                        .to_string(),
                ),
            },
        );

        // Card template
        self.templates.insert(
            "card".to_string(),
            Template {
                name: "Card".to_string(),
                category: "card".to_string(),
                html: r##"<article class="card" aria-labelledby="card-title">
  <img src="placeholder.jpg" alt="Card image" class="card__image" loading="lazy">
  <div class="card__content">
    <h3 id="card-title" class="card__title">Card Title</h3>
    <p class="card__description">Card description goes here with relevant content.</p>
    <a href="#" class="card__link" aria-label="Read more about Card Title">Read more</a>
  </div>
</article>"##
                    .to_string(),
                css: r#".card {
  --card-radius: 0.75rem;
  --card-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);

  display: flex;
  flex-direction: column;
  max-width: 20rem;
  background: var(--color-surface, #ffffff);
  border-radius: var(--card-radius);
  box-shadow: var(--card-shadow);
  overflow: hidden;
  transition: transform 0.2s, box-shadow 0.2s;
}

.card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.15);
}

.card__image {
  width: 100%;
  height: 12rem;
  object-fit: cover;
}

.card__content {
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.card__title {
  margin: 0;
  font-size: 1.25rem;
  color: var(--color-text, #1f2937);
}

.card__description {
  margin: 0;
  color: var(--color-text-muted, #6b7280);
  line-height: 1.5;
}

.card__link {
  color: var(--color-primary, #3b82f6);
  text-decoration: none;
  font-weight: 500;
}

.card__link:hover {
  text-decoration: underline;
}

.card__link:focus {
  outline: 2px solid var(--color-primary, #3b82f6);
  outline-offset: 2px;
}"#
                    .to_string(),
                javascript: None,
            },
        );

        // Navbar template
        self.templates.insert(
            "navbar".to_string(),
            Template {
                name: "Navbar".to_string(),
                category: "navbar".to_string(),
                html: r##"<header class="navbar" role="banner">
  <nav class="navbar__container" aria-label="Main navigation">
    <a href="/" class="navbar__logo" aria-label="Home">
      <span class="navbar__logo-text">Brand</span>
    </a>

    <button class="navbar__toggle" aria-expanded="false" aria-controls="navbar-menu" aria-label="Toggle navigation">
      <span class="navbar__toggle-icon"></span>
    </button>

    <ul id="navbar-menu" class="navbar__menu" role="menubar">
      <li class="navbar__item" role="none">
        <a href="#" class="navbar__link" role="menuitem">Home</a>
      </li>
      <li class="navbar__item" role="none">
        <a href="#" class="navbar__link" role="menuitem">About</a>
      </li>
      <li class="navbar__item" role="none">
        <a href="#" class="navbar__link" role="menuitem">Services</a>
      </li>
      <li class="navbar__item" role="none">
        <a href="#" class="navbar__link" role="menuitem">Contact</a>
      </li>
    </ul>
  </nav>
</header>"##
                    .to_string(),
                css: r#".navbar {
  --navbar-height: 4rem;
  --navbar-bg: var(--color-surface, #ffffff);

  position: sticky;
  top: 0;
  z-index: 100;
  background: var(--navbar-bg);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.navbar__container {
  display: flex;
  align-items: center;
  justify-content: space-between;
  max-width: 80rem;
  height: var(--navbar-height);
  margin: 0 auto;
  padding: 0 1.5rem;
}

.navbar__logo {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-text, #1f2937);
  text-decoration: none;
}

.navbar__toggle {
  display: none;
  padding: 0.5rem;
  background: none;
  border: none;
  cursor: pointer;
}

.navbar__toggle-icon {
  display: block;
  width: 1.5rem;
  height: 2px;
  background: var(--color-text, #1f2937);
  position: relative;
}

.navbar__toggle-icon::before,
.navbar__toggle-icon::after {
  content: '';
  position: absolute;
  width: 100%;
  height: 2px;
  background: inherit;
  left: 0;
}

.navbar__toggle-icon::before { top: -6px; }
.navbar__toggle-icon::after { bottom: -6px; }

.navbar__menu {
  display: flex;
  gap: 2rem;
  list-style: none;
  margin: 0;
  padding: 0;
}

.navbar__link {
  color: var(--color-text-muted, #6b7280);
  text-decoration: none;
  font-weight: 500;
  transition: color 0.2s;
}

.navbar__link:hover,
.navbar__link:focus {
  color: var(--color-primary, #3b82f6);
}

/* Mobile styles */
@media (max-width: 768px) {
  .navbar__toggle {
    display: block;
  }

  .navbar__menu {
    display: none;
    position: absolute;
    top: var(--navbar-height);
    left: 0;
    right: 0;
    flex-direction: column;
    gap: 0;
    background: var(--navbar-bg);
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .navbar__menu.is-open {
    display: flex;
  }

  .navbar__link {
    display: block;
    padding: 1rem 1.5rem;
  }
}"#
                    .to_string(),
                javascript: Some(
                    r#"const toggle = document.querySelector('.navbar__toggle');
const menu = document.querySelector('.navbar__menu');

toggle.addEventListener('click', () => {
  const isOpen = toggle.getAttribute('aria-expanded') === 'true';
  toggle.setAttribute('aria-expanded', !isOpen);
  menu.classList.toggle('is-open');
});"#
                        .to_string(),
                ),
            },
        );
    }
}
