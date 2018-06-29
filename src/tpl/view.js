$("#edit").click(function(){
    $("#view_tab").hide();
    $("#edit_tab").show();
});

$("#save").click(function(){
    $("#view_tab").show();
    $("#edit_tab").hide();
});

$(document).ready(function() {
  var editor = ace.edit("editor");
  editor.setTheme("ace/theme/textmate");
  editor.session.setMode("ace/mode/markdown");
  editor.setKeyboardHandler("ace/keyboard/vim");
});
