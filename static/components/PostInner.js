export default Vue.component('post-inner', {
  template: '<div></div>',
  props: { html: String },
  mounted() {
    const res = Vue.compile(`<div>${this.html}</div>`);
    const { render, staticRenderFns } = res;
    return new Vue({ el: this.$el, render, staticRenderFns });
  },
});
