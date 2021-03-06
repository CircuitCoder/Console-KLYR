import tmpl from './Post.html';
import { request } from '../util';

export default Vue.component('Post', {
  template: tmpl,
  props: ['user'],

  data: () => ({
    content: null,
    compiled: '',
    failed: false,
    rejection: false,
    notes: '',
  }),

  created() {
    this.grab();
  },

  methods: {
    async grab() {
      this.content = null;
      this.failed = false;

      let url = `/api/posts/${this.$route.params.id}`;
      if(this.$route.query.pending) url += '?pending';

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

    async accept() {
      const url = `/api/posts/${this.content.id}/accept`;
      const resp = await request(url, 'PUT', {});

      if(resp.ok) {
        this.$root.$emit('show-snackbar', {
          message: 'Done. You may backlog the message now.',
        });
      } else {
        this.$root.$emit('show-snackbar', {
          message: 'Failed',
        });
      }
    },

    reject() {
      this.rejection = true;
    },

    async sendReject() {
      const url = `/api/posts/${this.$route.params.id}/reject`;
      const resp = await request(url, 'PUT', { comment: this.notes });
      if(resp.ok) {
        this.$root.$emit('show-snackbar', {
          message: 'Done. You may backlog the message now.',
        });
      } else {
        this.$root.$emit('show-snackbar', {
          message: 'Failed',
        });
      }
    },

    edit() {
      this.$router.push({
        name: 'EditPost',
        params: { id: this.$route.params.id },
      });
    },

    formatTime(t) {
      /* global moment */
      return moment(t * 1000).format('MM/DD HH:mm:ss');
    },
  },

  watch: {
    $route() {
      this.grab();
    },
  },

  computed: {
    canEdit() {
      if(!this.user) return false;
      return this.content !== null && this.$route.query.pending && this.user.id === this.content.author;
    },

    canManage() {
      if(!this.user) return false;
      return this.content !== null && this.$route.query.pending && this.user.groups.includes('reviewers');
    },
  },
});
