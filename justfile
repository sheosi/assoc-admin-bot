chromedriver:
    mkdir -p vendor && \
    wget https://storage.googleapis.com/chrome-for-testing-public/142.0.7444.61/linux64/chromedriver-linux64.zip -O vendor/chromedriver-linux64.zip && \
    wget https://storage.googleapis.com/chrome-for-testing-public/142.0.7444.61/linux64/chrome-linux64.zip -O vendor/chrome-linux64.zip && \
    unzip vendor/chromedriver-linux64.zip -d vendor && \
    unzip vendor/chrome-linux64.zip -d vendor && \
    rm vendor/chromedriver-linux64.zip && \
    rm vendor/chrome-linux64.zip && \
    sudo mv vendor/chromedriver-linux64/chromedriver /usr/local/bin && \
    sudo mv vendor/chrome-linux64 /opt/chrome && \
    sudo ln -s /opt/chrome/bin/chrome /usr/local/bin/chrome && \
    rm -rf vendor && \
