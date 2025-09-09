from setuptools import setup, find_packages

setup(
    name="portify",
    version="1.0.0",
    description="A lightweight CLI tool for developers to manage ports and processes",
    author="Diego Fagundez",
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
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Operating System :: OS Independent",
    ],
)
