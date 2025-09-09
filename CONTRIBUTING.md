# Contributing to Portify

Thank you for your interest in contributing to Portify! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites
- Python 3.8 or higher
- macOS (for menu bar app development)
- Git

### Development Setup

1. **Fork and clone the repository**
   ```bash
   git clone https://github.com/your-username/portify.git
   cd portify
   ```

2. **Create a virtual environment**
   ```bash
   python -m venv venv
   source venv/bin/activate  # On Windows: venv\Scripts\activate
   ```

3. **Install dependencies**
   ```bash
   pip install -r requirements.txt
   pip install -r requirements-menubar.txt
   pip install -e .
   ```

4. **Test the installation**
   ```bash
   portify --help
   portify list
   ```

## 🛠️ Development Workflow

### Making Changes

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Follow the existing code style
   - Add docstrings to new functions
   - Update documentation if needed

3. **Test your changes**
   ```bash
   # Test CLI functionality
   portify list
   portify kill --help
   
   # Test menu bar app (macOS only)
   portify menubar
   ```

4. **Commit your changes**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

### Commit Message Format

We use conventional commits:
- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `style:` - Code style changes
- `refactor:` - Code refactoring
- `test:` - Adding tests
- `chore:` - Maintenance tasks

## 📝 Code Style

### Python Style Guidelines
- Follow PEP 8
- Use type hints where appropriate
- Maximum line length: 88 characters
- Use descriptive variable names

### Example:
```python
def scan_ports(self, include_system_info: bool = False) -> List[PortInfo]:
    """
    Scan all active network connections and return port information.
    
    Args:
        include_system_info: Whether to include CPU and memory usage
        
    Returns:
        List of PortInfo objects
    """
    # Implementation here
```

## 🧪 Testing

### Manual Testing
- Test CLI commands on different systems
- Test menu bar app functionality
- Verify error handling

### Adding Tests
We welcome test contributions! Tests should be added to a `tests/` directory.

## 📚 Documentation

### Updating Documentation
- Update README.md for user-facing changes
- Update docstrings for code changes
- Add examples for new features

### Documentation Structure
```
docs/
├── CLIENT_INSTALL_GUIDE.md    # End-user installation
├── MENUBAR_GUIDE.md           # Menu bar app guide
├── DEVELOPMENT_WORKFLOW.md    # Development process
└── DISTRIBUTION.md            # Distribution guide
```

## 🐛 Bug Reports

### Before Submitting
1. Check existing issues
2. Test with the latest version
3. Gather system information

### Bug Report Template
```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Run command '...'
2. See error

**Expected behavior**
What you expected to happen.

**System Information:**
- OS: [e.g. macOS 14.0]
- Python version: [e.g. 3.11.0]
- Portify version: [e.g. 1.0.0]

**Additional context**
Any other context about the problem.
```

## 💡 Feature Requests

We welcome feature requests! Please:
1. Check existing issues first
2. Describe the use case
3. Explain why it would be useful
4. Consider implementation complexity

## 🔄 Pull Request Process

1. **Ensure your PR:**
   - Has a clear description
   - References any related issues
   - Includes tests if applicable
   - Updates documentation if needed

2. **PR will be reviewed for:**
   - Code quality and style
   - Functionality and correctness
   - Documentation completeness
   - Compatibility

3. **After approval:**
   - PR will be merged to main
   - Changes will be included in next release

## 🏗️ Building and Distribution

### Building macOS App
```bash
python scripts/create_app_icon.py
python scripts/create_macos_app.py
```

### Creating Release
1. Update version in `portify/__init__.py` and `setup.py`
2. Update `CHANGELOG.md`
3. Create and push tag: `git tag v1.x.x && git push origin v1.x.x`
4. GitHub Actions will automatically build and create release

## 📞 Getting Help

- 💬 **Discussions**: Use GitHub Discussions for questions
- 🐛 **Issues**: Use GitHub Issues for bugs and feature requests
- 📧 **Email**: Contact maintainers for security issues

## 📄 License

By contributing to Portify, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Portify! 🚀
