import tmpl from './TreeView.html';

export default Vue.component('TreeView', {
  template: tmpl,
  props: ['steps'],
  data: () => ({
    reg: {},
  }),

  methods: {
    formatTime(t) {
      /* global moment */
      return moment(t * 1000).format('MM/DD HH:mm:ss');
    },

    toggle(id) {
      this.$set(this.reg, id, !this.reg[id]);
    },
  },
});
