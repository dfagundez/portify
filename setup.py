from setuptools import setup, find_packages

setup(
    name="portify",
    version="1.0.0",
    description="The fastest way to manage ports and processes on macOS - Always accessible from your menu bar",
    long_description="Portify is a macOS menu bar application that gives developers instant access to port and process management. No more switching to terminal - just click the menu bar icon to see active ports and kill processes with one click. Also includes a powerful CLI for advanced users.",
    long_description_content_type="text/markdown",
    author="Diego Fagundez",
    url="https://github.com/dfagundez/portify",
    packages=find_packages(),
    install_requires=[
        "typer[all]==0.9.0",
        "rich==13.7.0",
        "psutil==5.9.6",
        "click==8.1.7",
    ],
    extras_require={
        "menubar": [
            "pystray==0.19.5",
            "Pillow==10.1.0",
            "plyer==2.1.0",
        ],
    },
    entry_points={
        "console_scripts": [
            "portify=portify.main:app",
        ],
    },
    python_requires=">=3.8",
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Operating System :: MacOS",
        "Operating System :: POSIX :: Linux",
        "Topic :: Software Development :: Debuggers",
        "Topic :: System :: Monitoring",
        "Topic :: System :: Networking :: Monitoring",
        "Topic :: Utilities",
    ],
    keywords="port management, process killer, menu bar, macOS, developer tools, network monitoring",
)
