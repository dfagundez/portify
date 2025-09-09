"""
Main entry point for Portify CLI application.
"""

import sys
from .cli.commands import app


def main():
    """Main entry point for the CLI application."""
    try:
        app()
    except KeyboardInterrupt:
        print("\n👋 Goodbye!")
        sys.exit(0)
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
