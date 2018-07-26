module.exports = {
  "extends": "airbnb-base",
  "env": {
    "browser": true,
  },

  globals: {
    Vue: true,
    VueRouter: true,
  },

  rules: {
    'keyword-spacing': ['error', {
      overrides: {
        "if": { "after": false },
        "for": { "after": false },
        "while": { "after": false },
        "catch": { "after": false },
      },
    }],
  },
};