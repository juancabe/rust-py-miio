python3.11 -m venv venv
source venv/bin/activate
rm -r target
pip install git+https://github.com/rytilahti/python-miio.git
export PYTHONPATH="$(pwd)/venv/lib/python3.11/site-packages"
RUST_TEST_THREADS=1 cargo test