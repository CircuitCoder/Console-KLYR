import tmpl from './Step.html';
import { request } from '../util';
import CONST from '../const';

export default Vue.component('Step', {
  template: tmpl,
  props: ['user'],

  data: () => ({
    content: null,
    compiled: '',
    failed: false,
    assignment: false,
    picking: false,
    notes: '',

    CONST,
  }),

  created() {
    this.grab();
  },

  methods: {
    async grab() {
      this.content = null;
      this.failed = false;

      let url = `/api/steps/${this.$route.params.id}`;
      if(this.$route.query.pending) url += '?pending';

      try {
        this.content = await request(url);

        /* global markdownit */
        this.compiled = markdownit().render(this.content.step.content)
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

    reply() {
      this.$router.push({
        name: 'ReplyStep',
        params: { id: this.$route.params.id },
      });
    },

    async resolve() {
      const url = `/api/steps/${this.$route.params.id}/resolve`;
      const resp = await request(url, 'POST', {});

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

    assign() {
      this.assignment = true;
    },

    async submitAssignment() {
      const result = [];
      this.$refs.chips.forEach((e, id) => {
        if(e.isSelected()) result.push(this.CONST.resolvers[id].id);
      });

      const url = `/api/steps/${this.$route.params.id}/assign`;
      const resp = await request(url, 'PUT', result);

      if(resp.ok) {
        this.$root.$emit('show-snackbar', {
          message: 'Done. You may backlog the message now.',
        });
        await this.grab();
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
    canReply() {
      if(!this.content) return false;
      if(!this.user) return false;
      if(!this.content.staged) return false;
      return this.content.assignees.includes(this.user.id);
    },

    canAssign() {
      if(!this.user) return false;
      if(!this.content) return false;
      if(!this.content.staged) return false;
      if(!this.user.groups.includes('coordinators')) return false;
      return this.content.assignees.length === 0;
    },
  },
});
