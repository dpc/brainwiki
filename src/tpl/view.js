var editor;

$("#edit").click(function(){
  $("#view_tab").hide();
  $("#edit_tab").show();
  editor.focus();
});

$("#save").click(function(){
  const text = editor.getValue();
  $.ajax({
    contentType: 'application/json',
    type: 'PUT',
    url: window.location.href,
    data: JSON.stringify({
      text: text
    }),
    success: function(data) {
      alert(data);

      $("#view_tab").show();
      $("#edit_tab").hide();
      editor.focus();
    },
    error:function(data) {
      if (data.status == 409) {
        alert('Conflict. Try more tags.');
      } else if (data.status == 404) {
        alert('Not found.');
      } else {
        alert('Unknown error. Status: ' + data.status);
      }
    editor.focus();
    },
    dataType: 'json'
  });
});

$(document).ready(function() {
  editor = ace.edit("editor");
  editor.setTheme("ace/theme/textmate");
  editor.session.setMode("ace/mode/markdown");
  editor.setKeyboardHandler("ace/keyboard/vim");
});
