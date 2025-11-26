all:
	rm -rf dist
	uv run maturin sdist --out dist

publish:
	uv publish

clean:
	rm -rf dist target .venv
