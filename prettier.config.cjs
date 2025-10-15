const config = {
  semi: true,
  trailingComma: 'all',
  useTabs: false,
  singleQuote: true,
  printWidth: 80,
  tabWidth: 2,
  overrides: [
    {
      files: '**/package.json',
      options: {
        parser: 'json',
        tabWidth: 6,
      },
    },
    {
      files: '**/*.json',
      options: {
        tabWidth: 6,
      },
    },
  ],
};

module.exports = config;
