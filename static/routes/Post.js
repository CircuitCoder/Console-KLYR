import tmpl from './Post.html';
import { request } from '../util';

export default Vue.component('Post', {
  template: tmpl,

  data: () => ({
    content: null,
    compiled: '',
    failed: false,
  }),

  created() {
    this.grab();
  },

  methods: {
    async grab() {
      this.content = null;
      this.failed = false;

      let url = `/api/posts/${this.$route.params.id}`;
      if('pending' in this.$route.query) url += '?pending';

      try {
        this.content = await request(url);

        /* global markdownit */
        this.compiled = markdownit().render(this.content.content)
          .replace(/<h1>/, '<mdc-headline>')
          .replace(/<\/h1>/, '</mdc-headline>')
          .replace(/<h2>/, '<mdc-title>')
          .replace(/<\/h2>/, '</mdc-title>')
          .replace(/<h3>/, '<mdc-subheading>')
          .replace(/<\/h3>/, '</mdc-subheading>')
          .replace(/<p>/, '<mdc-body>')
          .replace(/<\/p>/, '</mdc-body>');
      } catch(e) {
        this.failed = true;
      }
    },
  },

  watch: {
    $route() {
      this.grab();
    },
  },
});
